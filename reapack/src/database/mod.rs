use crate::model::{
    InstalledFile, InstalledPackage, InstalledPackageType, InstalledVersionName, LightPackageId,
    PackageType, Section, VersionName,
};

use enumset::EnumSet;
use num_enum::TryFromPrimitive;
use ormlite::model::Insertable;
use ormlite::sqlite::{Sqlite, SqliteConnectOptions, SqliteConnection};
use ormlite::{Connection, Model};

use sqlx::Transaction;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::ops::DerefMut;
use std::path::Path;
use std::str::FromStr;
use tracing::instrument;

/// This is the currently supported ReaPack database user version.
///
/// For ReaBoot, this version number has the following meaning:
///
/// - If the database doesn't exist yet, it will create a new database with that user version and
///   a database schema corresponding to that version. If the latest ReaPack version is
///   made for a newer DB user version, it's still okay. ReaPack will carry out the necessary
///   migration. If ReaPack raises the **major** DB user version in the process, ReaBoot will not be
///   usable *after* that initial installation (see below).
/// - If the database exists already and its **major** user version is *greater* than the one
///   defined here, ReaBoot will refuse to continue because the database schema turned incompatible.
///   In that case, it's a good idea to prompt the user to check if a new ReaBoot version
///   is available.
/// - If the database exists already and its user version is *lower* than the one defined here,
///   ReaBoot will apply the same DB migrations that ReaPack would do and raise the DB user version
///   to the one defined here.
///
/// This version number and a few other things should be adjusted whenever ReaPack raises it. If
/// it happens a bit later, no problem. The only issue with an out-of-date ReaBoot is that it
/// refuses operation if the user has a ReaPack database with a higher major version.
///
/// Whenever ReaPack raises the version, update the following things:
///
/// - [`REAPACK_DB_USER_VERSION`]
/// - [`DatabaseTransaction::init`] (`sql/create-tables.sql`)
/// - [`DatabaseTransaction::migrate_from`]
///
/// See https://github.com/cfillion/reapack/blob/master/src/registry.cpp (method `migrate`).
pub const REAPACK_DB_USER_VERSION: DbUserVersion = DbUserVersion { major: 0, minor: 6 };

#[derive(Debug)]
pub struct Database {
    connection: SqliteConnection,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct DbUserVersion {
    // MUST COME FIRST (influences derived ordering)
    pub major: i16,
    // MUST COME SECOND (influences derived ordering)
    pub minor: i16,
}

impl DbUserVersion {
    pub const UNINITIALIZED: Self = Self { major: 0, minor: 0 };

    pub fn from_raw(raw: i32) -> Self {
        Self {
            major: (raw >> 16) as i16,
            minor: raw as i16,
        }
    }

    pub fn to_raw(&self) -> i32 {
        self.minor as i32 | ((self.major as i32) << 16)
    }
}

impl Database {
    /// Creates the database file, creates tables and sets the initial user version.
    ///
    /// It's important that we do the same here as the original ReaPack
    /// does in the ReaPack version corresponding to the [`REAPACK_DB_USER_VERSION`].
    ///
    /// https://github.com/cfillion/reapack/blob/master/src/registry.cpp
    pub async fn create(db_file: impl AsRef<Path>) -> anyhow::Result<Self> {
        // Create parent folders
        let db_file = db_file.as_ref();
        if let Some(parent) = db_file.parent() {
            fs::create_dir_all(parent)?;
        }
        // Open and init DB
        let mut db = Self::new(db_file, true).await?;
        db.init().await?;
        Ok(db)
    }

    /// Opens the given database file.
    ///
    /// Unlike in the original ReaPack, this doesn't include migration! It's a separate method.
    /// For ReaBoot, that's better because it's more of a "one shot" application. We potentially
    /// connect multiple times during the execution of ReaBoot in order to not having to hold
    /// on to files in an async context.
    pub async fn open(db_file: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::new(db_file, false).await
    }

    /// Closes the database, making sure that everything has been written to disk when the future
    /// is finished.
    #[instrument]
    pub async fn close(self) -> anyhow::Result<()> {
        self.connection.close().await?;
        Ok(())
    }

    async fn new(db_file: impl AsRef<Path>, create_if_missing: bool) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(db_file)
            .pragma("foreign_keys", "1")
            .create_if_missing(create_if_missing);
        let connection = SqliteConnection::connect_with(&options).await?;
        let db = Self { connection };
        Ok(db)
    }

    pub async fn installed_packages(&mut self) -> anyhow::Result<Vec<InstalledPackage>> {
        let entries = self.entries().await?;
        let files = self.files().await?;
        let mut files_by_entry_id: HashMap<i32, Vec<DbFile>> = HashMap::new();
        for f in files {
            files_by_entry_id.entry(f.entry).or_default().push(f);
        }
        let installed_packages = entries
            .into_iter()
            .map(|entry| {
                let db_files = files_by_entry_id.remove(&entry.id).unwrap_or_default();
                convert_db_entry_to_model(entry, db_files)
            })
            .collect();
        Ok(installed_packages)
    }

    pub async fn user_version(&mut self) -> anyhow::Result<DbUserVersion> {
        let (raw,): (i32,) = ormlite::query_as("PRAGMA user_version")
            .fetch_one(&mut self.connection)
            .await?;
        Ok(DbUserVersion::from_raw(raw))
    }

    pub async fn entries(&mut self) -> anyhow::Result<Vec<DbEntry>> {
        Ok(DbEntry::select().fetch_all(&mut self.connection).await?)
    }

    pub async fn files(&mut self) -> anyhow::Result<Vec<DbFile>> {
        Ok(DbFile::select().fetch_all(&mut self.connection).await?)
    }

    pub async fn with_transaction<'a, 'b, F>(
        &'a mut self,
        f: impl FnOnce(DatabaseTransaction<'b>) -> F + 'b,
    ) -> anyhow::Result<()>
    where
        F: Future<Output = anyhow::Result<DatabaseTransaction<'b>>> + 'b,
        'a: 'b,
    {
        let internal_transaction = self.connection.begin().await?;
        let transaction = f(DatabaseTransaction(internal_transaction)).await?;
        transaction.0.commit().await?;
        Ok(())
    }

    pub async fn compatibility_info(&mut self) -> anyhow::Result<CompatibilityInfo> {
        let v = self.user_version().await?;
        let info = match v.cmp(&REAPACK_DB_USER_VERSION) {
            Ordering::Less => CompatibilityInfo::CompatibleButNeedsMigration,
            Ordering::Equal => CompatibilityInfo::PerfectlyCompatible,
            Ordering::Greater => {
                if v.major > REAPACK_DB_USER_VERSION.major {
                    // The major version is higher! Oh no!
                    CompatibilityInfo::DbTooNew
                } else {
                    CompatibilityInfo::DbNewerButCompatible
                }
            }
        };
        Ok(info)
    }

    /// Creates tables and sets the initial user version.
    #[instrument]
    async fn init(&mut self) -> anyhow::Result<()> {
        let transaction = self.with_transaction(|mut t| async {
            t.init().await?;
            Ok(t)
        });
        transaction.await?;
        Ok(())
    }

    /// Migrates from an older DB version if necessary.
    #[instrument]
    pub async fn migrate(&mut self) -> anyhow::Result<()> {
        let v = self.user_version().await?;
        if v > REAPACK_DB_USER_VERSION {
            // Version of DB is greater than the version this ReaBoot version was made for.
            return Ok(());
        }
        if v == DbUserVersion::UNINITIALIZED {
            // This shouldn't happen in normal operation. The database exists but was not
            // initialized yet by ReaBoot or ReaPack. Do a full initialization instead of gradual
            // migration.
            self.init().await?;
            return Ok(());
        }
        // Starting from here, we for sure do some migration.
        let transaction = self.with_transaction(|mut t| async {
            t.migrate_from(v).await?;
            Ok(t)
        });
        transaction.await?;
        Ok(())
    }
}

/// Compatibility of this ReaBoot version with a given ReaPack registry database.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CompatibilityInfo {
    /// Versions match exactly. Jackpot.
    PerfectlyCompatible,
    /// DB version is older than the version this ReaBoot version was made for. No problem, it
    /// just needs migration.
    CompatibleButNeedsMigration,
    /// DB version is greater than the version this ReaBoot version was made for, but it's
    /// still backward-compatible.
    ///
    /// We can still operate, we just might not use all the new features.
    DbNewerButCompatible,
    /// DB version is newer than the version this ReaBoot version was made for and not
    /// backward-compatible anymore.
    ///
    /// Operation is not possible.
    DbTooNew,
}

pub struct DatabaseTransaction<'a>(Transaction<'a, Sqlite>);

impl<'a> DatabaseTransaction<'a> {
    pub async fn set_user_version(&mut self, version: DbUserVersion) -> anyhow::Result<()> {
        ormlite::query(&format!("PRAGMA user_version = {}", version.to_raw()))
            .execute(self.0.deref_mut())
            .await?;
        Ok(())
    }

    pub async fn remove_package(&mut self, package_id: LightPackageId<'_>) -> anyhow::Result<()> {
        // ReaPack's database schema has a foreign key defined, but without cascade delete. So
        // we need to manually remove all files and then the entry.
        // Get entry id
        let id_query = "SELECT id FROM entries WHERE remote = ? AND category = ? AND package = ?";
        let (id,): (i32,) = ormlite::query_as(id_query)
            .bind(package_id.remote.to_string())
            .bind(package_id.category.to_string())
            .bind(package_id.package.to_string())
            .fetch_one(self.0.deref_mut())
            .await?;
        // Delete files
        let delete_files_query = "DELETE FROM files WHERE entry = ?";
        ormlite::query(delete_files_query)
            .bind(id)
            .execute(self.0.deref_mut())
            .await?;
        // Delete entry
        let delete_entry_query = "DELETE FROM entries WHERE id = ?";
        ormlite::query(delete_entry_query)
            .bind(id)
            .execute(self.0.deref_mut())
            .await?;
        Ok(())
    }

    pub async fn add_package(&mut self, package: InstalledPackage) -> anyhow::Result<()> {
        let entry = InsertDbEntry {
            remote: package.remote,
            category: package.category,
            package: package.package,
            desc: package.desc,
            typ: convert_model_package_type_to_db(package.typ),
            version: convert_model_version_name_to_db(package.version),
            author: package.author,
            // We don't care about flags at the moment and use the defaults
            flags: Some(0),
        };
        let entry = entry.insert(&mut self.0).await?;
        for f in package.files {
            let file = InsertDbFile {
                entry: entry.id,
                path: f.path,
                main: convert_model_sections_to_db(f.sections),
                typ: f.typ.map(convert_model_package_type_to_db).unwrap_or(0),
            };
            file.insert(&mut self.0).await?;
        }
        Ok(())
    }

    /// This must conform to ReaPack's initialization.
    /// See https://github.com/cfillion/reapack/blob/master/src/registry.cpp (method `migrate`)
    async fn init(&mut self) -> anyhow::Result<()> {
        ormlite::query(include_str!("sql/create-tables.sql"))
            .execute(self.0.deref_mut())
            .await?;
        self.set_user_version(REAPACK_DB_USER_VERSION).await?;
        Ok(())
    }

    /// This must conform to ReaPack's migration.
    /// See https://github.com/cfillion/reapack/blob/master/src/registry.cpp (method `migrate`)
    async fn migrate_from(&mut self, v: DbUserVersion) -> anyhow::Result<()> {
        let t = (v.major, v.minor);
        if t <= (0, 1) {
            self.exec("ALTER TABLE entries ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;")
                .await?;
        }
        if t <= (0, 2) {
            self.exec("ALTER TABLE files ADD COLUMN type INTEGER NOT NULL DEFAULT 0;")
                .await?;
        }
        if t <= (0, 3) {
            self.exec("ALTER TABLE entries ADD COLUMN desc TEXT NOT NULL DEFAULT '';")
                .await?;
        }
        if t <= (0, 4) {
            self.convert_implicit_sections().await?;
        }
        if t <= (0, 5) {
            // This was actually a backward-incompatible change
            self.exec("ALTER TABLE entries RENAME COLUMN pinned TO flags;")
                .await?;
        }
        self.set_user_version(v).await?;
        Ok(())
    }

    async fn convert_implicit_sections(&mut self) -> anyhow::Result<()> {
        // convert from v1.0 main=true format to v1.1 flag format
        let entries: Vec<(i32, String)> = ormlite::query_as("SELECT id, category FROM entries")
            .fetch_all(self.0.deref_mut())
            .await?;
        for (id, category) in entries {
            let section = Section::detect_from_category_legacy(category.as_ref());
            let section_string = section.to_string();
            ormlite::query("UPDATE files SET main = ? WHERE entry = ? AND main != 0")
                .bind(section_string)
                .bind(id)
                .execute(self.0.deref_mut())
                .await?;
        }
        Ok(())
    }

    async fn exec(&mut self, query: &str) -> anyhow::Result<()> {
        ormlite::query(query).execute(self.0.deref_mut()).await?;
        Ok(())
    }
}

#[derive(Model, Debug)]
#[ormlite(table = "entries")]
#[ormlite(insertable = InsertDbEntry)]
pub struct DbEntry {
    #[ormlite(primary_key)]
    pub id: i32,
    pub remote: String,
    pub category: String,
    pub package: String,
    pub desc: String,
    /// This should be != 0 because the package type must be set on entry level.
    #[ormlite(column = "type")]
    pub typ: i32,
    pub version: String,
    pub author: String,
    /// Bit flags for "pinned" and "bleeding edge".
    ///
    /// Irrelevant for us at the moment.
    pub flags: Option<i32>,
}

#[derive(Model, Debug)]
#[ormlite(table = "files")]
#[ormlite(insertable = InsertDbFile)]
pub struct DbFile {
    #[ormlite(primary_key)]
    pub id: i32,
    pub entry: i32,
    pub path: String,
    /// This is conceptually a set of sections represented as bit flags.
    ///
    /// - Value -1 means implicit section (corresponds value "true" in index XML).
    /// - Value 0 means unknown section = no sections
    pub main: i32,
    /// This can be 0 because it's just an override and 0 corresponds to "no override".
    #[ormlite(column = "type")]
    pub typ: i32,
}

fn convert_db_entry_to_model(entry: DbEntry, db_files: Vec<DbFile>) -> InstalledPackage {
    let files = db_files.into_iter().map(convert_db_file_to_model).collect();
    InstalledPackage {
        remote: entry.remote,
        category: entry.category,
        package: entry.package,
        desc: entry.desc,
        typ: convert_db_package_type_to_model(entry.typ),
        version: convert_db_version_name_to_model(entry.version),
        author: entry.author,
        files,
    }
}

fn convert_db_file_to_model(f: DbFile) -> InstalledFile {
    InstalledFile {
        path: f.path,
        sections: convert_db_sections_to_model(f.main),
        typ: if f.typ == 0 {
            None
        } else {
            Some(convert_db_package_type_to_model(f.typ))
        },
    }
}

fn convert_db_package_type_to_model(value: i32) -> InstalledPackageType {
    if let Ok(s) = PackageType::try_from_primitive(value) {
        InstalledPackageType::Known(s)
    } else {
        InstalledPackageType::Unknown(value)
    }
}

fn convert_db_version_name_to_model(value: String) -> InstalledVersionName {
    if let Ok(n) = VersionName::from_str(&value) {
        InstalledVersionName::Valid(n)
    } else {
        InstalledVersionName::Invalid(value)
    }
}

fn convert_model_version_name_to_db(value: InstalledVersionName) -> String {
    match value {
        InstalledVersionName::Valid(s) => s.to_string(),
        InstalledVersionName::Invalid(s) => s,
    }
}

fn convert_model_sections_to_db(value: Option<EnumSet<Section>>) -> i32 {
    value.map(|v| v.as_u32() as i32).unwrap_or(-1)
}

fn convert_db_sections_to_model(value: i32) -> Option<EnumSet<Section>> {
    if value < 0 {
        // -1 stands for implicit section = `None` in the model
        None
    } else {
        // We truncate 1. because having more than 32 sections is unlikely, and 2. because
        // the section is irrelevant for ReaBoot when reading from the database (unlike ReaPack, it
        // doesn't register actions at runtime, it just wants to know about locations of installed
        // files).
        Some(EnumSet::from_u32_truncated(value as u32))
    }
}

fn convert_model_package_type_to_db(value: InstalledPackageType) -> i32 {
    match value {
        InstalledPackageType::Known(t) => t as i32,
        InstalledPackageType::Unknown(i) => i,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn create_db() {
    //     let db_file = "/Users/helgoboss/Downloads/bla/foo/test-registry.db";
    //     let mut db = Database::create(db_file).await.unwrap();
    //     let entries = db.entries().await.unwrap();
    //     let files = db.files().await.unwrap();
    //     dbg!(entries, files);
    // }

    #[tokio::test]
    async fn open_db() {
        let mut db = Database::open("src/database/test/registry.db")
            .await
            .unwrap();
        let entries = db.entries().await.unwrap();
        let files = db.files().await.unwrap();
        dbg!(entries, files);
    }
}
