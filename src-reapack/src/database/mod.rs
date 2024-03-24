use crate::model::{
    InstalledFile, InstalledPackage, InstalledPackageType, InstalledVersionName, LightPackageId,
    PackageType, Section, VersionName,
};
use anyhow::{bail, Context};
use enumset::EnumSet;
use num_enum::TryFromPrimitive;
use ormlite::model::Insertable;
use ormlite::sqlite::{Sqlite, SqliteConnectOptions, SqliteConnection, SqliteTypeInfo};
use ormlite::{Connection, Model};
use serde::Deserialize;
use sqlx::Transaction;
use std::collections::HashMap;
use std::future::Future;
use std::ops::DerefMut;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use tokio::fs;

pub struct Database {
    connection: SqliteConnection,
}

impl Database {
    pub async fn create(db_file: impl AsRef<Path>) -> anyhow::Result<Self> {
        // Create parent folders
        let db_file = db_file.as_ref();
        if let Some(parent) = db_file.parent() {
            fs::create_dir_all(parent).await?;
        }
        // Open database
        let options = SqliteConnectOptions::new()
            .filename(db_file)
            .create_if_missing(true);
        let mut connection = SqliteConnection::connect_with(&options).await?;
        ormlite::query(include_str!("sql/create-tables.sql"))
            .execute(&mut connection)
            .await?;
        let db = Self { connection };
        Ok(db)
    }

    pub async fn open(db_file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(db_file)
            .create_if_missing(false);
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
}

pub struct DatabaseTransaction<'a>(Transaction<'a, Sqlite>);

impl<'a> DatabaseTransaction<'a> {
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
            flags: None,
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
        let mut db = Database::open(
            "/Users/helgoboss/Library/Application Support/REAPER/ReaPack/registry.db",
        )
        .await
        .unwrap();
        let entries = db.entries().await.unwrap();
        let files = db.files().await.unwrap();
        dbg!(entries, files);
    }
}
