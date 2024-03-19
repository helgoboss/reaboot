use crate::model::{InstalledPackage, PackageType, VersionName};
use anyhow::{bail, Context};
use camino::Utf8Path;
use ormlite::model::Insertable;
use ormlite::sqlite::{Sqlite, SqliteConnectOptions, SqliteConnection, SqliteTypeInfo};
use ormlite::{Connection, Model};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

pub struct Database {
    connection: SqliteConnection,
}

impl Database {
    pub async fn create(db_file: impl AsRef<Utf8Path>) -> anyhow::Result<Self> {
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

    pub async fn open(url: &str) -> anyhow::Result<Self> {
        let connection = SqliteConnection::connect(url).await?;
        let db = Self { connection };
        Ok(db)
    }

    pub async fn entries(&mut self) -> anyhow::Result<Vec<DbEntry>> {
        Ok(DbEntry::select().fetch_all(&mut self.connection).await?)
    }

    pub async fn files(&mut self) -> anyhow::Result<Vec<DbFile>> {
        Ok(DbFile::select().fetch_all(&mut self.connection).await?)
    }

    pub async fn add_package(&mut self, package: InstalledPackage) -> anyhow::Result<()> {
        let mut transaction = self.connection.begin().await?;
        let entry = InsertDbEntry {
            remote: package.remote,
            category: package.category,
            package: package.package,
            desc: package.desc,
            typ: package.typ as i32,
            version: package.version.to_string(),
            author: package.author,
            flags: None,
        };
        let entry = entry.insert(&mut transaction).await?;
        for f in package.files {
            let file = InsertDbFile {
                entry: entry.id,
                path: f.path,
                main: f.section.map(|s| s as i32).unwrap_or(0),
                typ: f.typ.map(|s| s as i32).unwrap_or(0),
            };
        }
        transaction.commit().await?;
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
    pub main: i32,
    #[ormlite(column = "type")]
    pub typ: i32,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
pub enum DbPackageType {
    Known(PackageType),
    Unknown(i32),
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
