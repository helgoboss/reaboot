use axum::http::StatusCode;
use fs_extra::dir::CopyOptions;
use reaboot_core::api::{ConfirmationRequest, InstallationStage, InstallerConfig};
use reaboot_core::installer::{InstallerListener, InstallerNewArgs, InstallerTask};
use reaboot_core::recipe::Recipe;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Column, Connection, Row, SqliteConnection, Value, ValueRef};
use std::fmt::{Debug, Display};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::spawn;
use tracing::instrument;

/// This integration test performs multiple test installations and checks if the resulting
/// REAPER resource directory has the correct contents.
///
/// No files are downloaded from the internet.
/// Instead, we start a local server that provides the repository index and referenced package
/// files.
///
/// We disable the download and installation of both ReaPack and REAPER.
/// We primarily check that the generated files are correct (most importantly, "reapack.ini"
/// and "ReaPack/registry.db").
#[test_log::test(tokio::test)]
async fn integration_test() {
    start_file_server(manifest_dir().join("tests/repository"), 56173);
    case_minimal().await;
    case_custom_package().await;
    case_recipe().await;
    case_package_exists_no_reapack().await;
    case_old_reapack().await;
}

/// If a file of a package exists already but hasn't been installed via ReaPack, ReaBoot should
/// not complain! It should take exclusive ownership of that file (until uninstalled) and overwrite
/// it. cfillion confirmed that this is the behavior of ReaPack.
async fn case_package_exists_no_reapack() {
    let case = TestCase {
        id: "package-exists-no-reapack",
        installation: "package-exists-no-reapack",
        recipe: Recipe::default(),
        package_urls: vec![format!(
            "http://localhost:56173/index.xml#p=Example/Hello%20World.lua&v=latest"
        )],
    };
    case.execute().await;
}

/// If an old ReaPack index exists, ReaBoot should migrate it exactly as a new ReaPack version would
/// do it. While migration was built into ReaBoot right from the start, there was a bug that
/// prevented it from happening: https://github.com/helgoboss/reaboot/issues/1
async fn case_old_reapack() {
    let case = TestCase {
        id: "old-reapack",
        installation: "old-reapack",
        recipe: Recipe::default(),
        package_urls: vec![format!(
            "http://localhost:56173/index.xml#p=Example/Hello%20World.lua&v=latest"
        )],
    };
    case.execute().await;
}

/// ReaBoot should be able to install a simple recipe.
async fn case_recipe() {
    let recipe = r#"
{
    "name": "Example",
    "required_packages": [
        "http://localhost:56173/index.xml#p=Example/Hello%20World.lua"
    ],
    "skip_additional_packages": false,
    "website": "https://www.example.com",
    "sub_title": "by Exampler"
}
"#;
    let case = TestCase {
        id: "recipe",
        installation: "vanilla",
        recipe: serde_json::from_str(recipe).unwrap(),
        package_urls: vec![],
    };
    case.execute().await;
}

/// ReaBoot should be able to install a simple custom package.
async fn case_custom_package() {
    let case = TestCase {
        id: "custom-package",
        installation: "vanilla",
        recipe: Recipe::default(),
        package_urls: vec![format!(
            "http://localhost:56173/index.xml#p=Example/Hello%20World.lua&v=latest"
        )],
    };
    case.execute().await;
}

/// ReaBoot should be able to run even if not package is provided.
///
/// In reality, this would still at the very least install/update ReaPack. And maybe even REAPER.
async fn case_minimal() {
    let case = TestCase {
        id: "minimal",
        installation: "vanilla",
        recipe: Recipe::default(),
        package_urls: vec![],
    };
    case.execute().await;
}

fn assert_dirs_equal_if_exist(dir1: &Path, dir2: &Path) {
    assert_dir_contains(dir1, dir2);
    assert_dir_contains(dir2, dir1);
}

fn assert_dir_contains(dir1: &Path, dir2: &Path) {
    if !dir1.exists() {
        // Main directory doesn't exist. That's also okay.
        return;
    }
    for entry1 in fs::read_dir(dir1).unwrap_or_else(|_| panic!("couldn't read directory {dir1:?}"))
    {
        let entry1 = entry1.unwrap();
        let entry1_path = entry1.path();
        let entry2_path = dir2.join(entry1.file_name());
        let entry1_is_dir = entry1.file_type().unwrap().is_dir();
        let entry2_is_dir = entry2_path.is_dir();
        if entry1_is_dir && entry2_is_dir {
            // Both are directories
            assert_dir_contains(&entry1_path, &entry2_path);
        } else if !entry1_is_dir && !entry2_is_dir {
            // Both are files
            assert_files_equal(&entry1_path, &entry2_path);
        } else {
            panic!("Directory entries {dir1:?} and {dir2:?} have different types");
        }
    }
}

fn assert_files_equal(path1: &Path, path2: &Path) {
    if assert_text_files_equal(path1, path2).is_err() {
        assert_binary_files_equal(path1, path2);
    }
}

fn assert_binary_files_equal(path1: &Path, path2: &Path) {
    let path1_bytes =
        std::fs::read(path1).unwrap_or_else(|_| panic!("File {path1:?} does not exist"));
    let path2_bytes =
        std::fs::read(path2).unwrap_or_else(|_| panic!("File {path2:?} does not exist"));
    assert_eq!(
        path1_bytes, path2_bytes,
        "Binary files {path1:?} and {path2:?} differ"
    );
}

/// # Errors
///
/// Returns an error if at least one of the file is not a valid UTF-8-encoded text file or doesn't
/// exist at all.
fn assert_text_files_equal(path1: &Path, path2: &Path) -> std::io::Result<()> {
    let path1_text = std::fs::read_to_string(path1)?;
    let path2_text = std::fs::read_to_string(path2)?;
    similar_asserts::assert_eq!(path1_text, path2_text);
    Ok(())
}

struct TestCase {
    id: &'static str,
    installation: &'static str,
    recipe: Recipe,
    package_urls: Vec<String>,
}

impl TestCase {
    async fn execute(self) -> ExecutedTestCase {
        println!("\n\n==== Executing test case [{}] ====\n", self.id);
        let manifest_dir = manifest_dir();
        let expected_cases_dir = manifest_dir.join("tests/cases");
        let expected_dir = expected_cases_dir.join(self.id);
        let src_installations_dir = manifest_dir.join("tests/installations");
        let src_installation_dir = src_installations_dir.join(self.installation);
        let target_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        let formatted_timestamp = jiff::Timestamp::now()
            .strftime("%Y-%m-%d_%H-%M-%S")
            .to_string();
        let target_test_dir = target_dir.join(&formatted_timestamp);
        let target_cases_dir = target_test_dir.join("cases");
        let actual_dir = target_cases_dir.join(self.id);
        // Copy desired installation to test directory
        fs_extra::copy_items(
            &[&src_installation_dir],
            &actual_dir,
            &CopyOptions {
                copy_inside: true,
                ..Default::default()
            },
        )
        .unwrap();
        // Install
        let config = InstallerConfig {
            custom_reaper_resource_dir: Some(actual_dir.clone()),
            package_urls: self.package_urls,
            recipe: Some(self.recipe),
            selected_features: Default::default(),
            install_reapack: Some(false),
            installation_id: Some("test".to_string()),
            ..Default::default()
        };
        let (_, interaction_receiver) = tokio::sync::broadcast::channel(10);
        let installer_new_args = InstallerNewArgs {
            config,
            temp_dir_for_reaper_download: target_dir.join("reaper"),
            interactions: interaction_receiver,
            listener: TestInstallerListener,
        };
        let installer = reaboot_core::installer::Installer::new(installer_new_args)
            .await
            .unwrap();
        let resolved_config = installer.resolved_config();
        assert!(resolved_config.reaper_is_installable);
        assert_eq!(resolved_config.reaper_exe_exists, true);
        assert_eq!(resolved_config.reaper_ini_exists, true);
        assert_eq!(resolved_config.portable, true);
        installer.install().await.unwrap();
        // Dump ReaPack registry.db to text (we don't want to compare binary DB files because
        // of OS differences)
        let registry_sql = dump_sqlite_database_as_sql(&actual_dir.join("ReaPack/registry.db"))
            .await
            .unwrap();
        fs::write(actual_dir.join("ReaPack/registry.sql"), registry_sql).unwrap();
        // Do basic assertions
        let executed = ExecutedTestCase {
            expected_dir,
            actual_dir,
        };
        executed.assert_dirs_equal_if_exist("ReaPack/Cache");
        executed.assert_dirs_equal_if_exist("ReaBoot");
        executed.assert_dirs_equal_if_exist("Scripts");
        executed.assert_files_equal("reapack.ini");
        executed.assert_files_equal("reaper.ini");
        executed.assert_files_equal("ReaPack/registry.sql");
        executed
    }
}

struct ExecutedTestCase {
    expected_dir: PathBuf,
    actual_dir: PathBuf,
}

impl ExecutedTestCase {
    fn assert_files_equal(&self, rel_path: &str) {
        assert_files_equal(
            &self.actual_dir.join(rel_path),
            &self.expected_dir.join(rel_path),
        );
    }

    fn assert_dirs_equal_if_exist(&self, rel_path: &str) {
        assert_dirs_equal_if_exist(
            &self.actual_dir.join(rel_path),
            &self.expected_dir.join(rel_path),
        );
    }
}

#[derive(Debug)]
struct TestInstallerListener;

impl InstallerListener for TestInstallerListener {
    #[instrument]
    fn installation_stage_changed(&self, _event: InstallationStage) {}

    fn installation_stage_progressed(&self, _progress: f64) {}

    #[instrument]
    fn task_started(&self, _task_id: u32, _task: InstallerTask) {}

    fn task_progressed(&self, _task_id: u32, _progress: f64) {}

    #[instrument]
    fn task_finished(&self, _task_id: u32) {}

    #[instrument]
    fn warn(&self, _message: impl Display + Debug) {}

    #[instrument]
    fn info(&self, _message: impl Display + Debug) {}

    #[instrument]
    fn debug(&self, _message: impl Display + Debug) {}

    #[instrument]
    fn confirm(&self, _request: ConfirmationRequest) {}
}

fn start_file_server(directory: impl AsRef<Path>, port: u16) {
    use axum::{routing::get_service, Router};
    use tower_http::services::ServeDir;
    let app = Router::new().nest_service(
        "/",
        get_service(ServeDir::new(directory))
            .handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "Error serving file") }),
    );

    let addr = ([127, 0, 0, 1], port).into();
    spawn(async move {
        axum_server::Server::bind(addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A very simplistic function converting the SQLite database at the given path to SQL.
///
/// This is used for asserting database contents.
async fn dump_sqlite_database_as_sql(db_file: &Path) -> anyhow::Result<String> {
    let options = SqliteConnectOptions::new()
        .filename(db_file)
        .create_if_missing(false);
    let mut con = SqliteConnection::connect_with(&options).await?;
    let mut dump = String::new();
    // Pragmas
    let pragmas = ["foreign_keys", "user_version"];
    for pragma in pragmas {
        use std::fmt::Write;

        let value: i32 = sqlx::query_scalar(&format!("PRAGMA {pragma};"))
            .fetch_one(&mut con)
            .await?;
        writeln!(&mut dump, "PRAGMA {pragma} = {value};").unwrap();
    }
    // Schema dump
    let schema_rows = sqlx::query("SELECT sql FROM sqlite_master WHERE type IN ('table', 'index', 'trigger') AND sql IS NOT NULL")
        .fetch_all(&mut con)
        .await?;
    for row in schema_rows {
        let sql: String = row.get("sql");
        dump.push_str(&sql);
        dump.push_str(";\n");
    }
    // Data dump for each table
    let table_rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(&mut con)
    .await?;
    for table_row in table_rows {
        let table_name: String = table_row.get("name");
        // Fetch rows from the table
        let query = format!("SELECT * FROM {}", table_name);
        let rows = sqlx::query(&query).fetch_all(&mut con).await?;
        if rows.is_empty() {
            continue;
        }
        let column_names: Vec<_> = rows[0]
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect();
        for row in rows {
            let values: Vec<String> = column_names
                .iter()
                .enumerate()
                .map(|(i, column_name)| {
                    let raw_value = row.try_get_raw(i).unwrap().to_owned();
                    if raw_value.is_null() {
                        "NULL".to_string()
                    } else if let Ok(v) = raw_value.try_decode::<String>() {
                        format!("'{}'", v.replace("'", "''"))
                    } else if let Ok(v) = raw_value.try_decode::<i64>() {
                        v.to_string()
                    } else {
                        panic!("Unknown type of raw value {} (table name {table_name}, column {column_name})", raw_value.type_info())
                    }
                })
                .collect();

            let insert_statement = format!(
                "INSERT INTO {} ({}) VALUES ({});\n",
                table_name,
                column_names.join(", "),
                values.join(", ")
            );
            dump.push_str(&insert_statement);
        }
    }
    Ok(dump)
}
