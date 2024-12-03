use fs_extra::dir::CopyOptions;
use reaboot_core::api::{ConfirmationRequest, InstallationStage, InstallerConfig};
use reaboot_core::installer::{InstallerListener, InstallerNewArgs, InstallerTask};
use reaboot_core::recipe::Recipe;
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use tracing::instrument;

#[test_log::test(tokio::test)]
async fn install_minimal() {
    let case = TestCase {
        id: "minimal",
        installation: "vanilla",
        recipe: Recipe::default(),
        package_urls: vec![],
    };
    case.execute().await;
}

fn assert_binary_files_equal(path1: &PathBuf, path2: &PathBuf) {
    let path1_bytes = std::fs::read(path1).unwrap();
    let path2_bytes = std::fs::read(path2).unwrap();
    assert_eq!(path1_bytes, path2_bytes);
}

fn assert_text_files_equal(path1: &PathBuf, path2: &PathBuf) {
    let path1_text = std::fs::read_to_string(path1).unwrap();
    let path2_text = std::fs::read_to_string(path2).unwrap();
    similar_asserts::assert_eq!(path1_text, path2_text);
}

struct TestCase {
    id: &'static str,
    installation: &'static str,
    recipe: Recipe,
    package_urls: Vec<String>,
}

impl TestCase {
    async fn execute(self) -> ExecutedTestCase {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let expected_cases_dir = manifest_dir.join("tests/cases");
        let expected_dir = expected_cases_dir.join(self.id);
        let src_installations_dir = manifest_dir.join("tests/installations");
        let src_installation_dir = src_installations_dir.join(self.installation);
        let target_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        let ts = jiff::Timestamp::now()
            .strftime("%Y-%m-%d_%H-%M-%S")
            .to_string();
        let target_test_dir = target_dir.join(ts);
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
        assert!(installer.reaper_is_installable());
        let resolved_config = installer.resolved_config();
        assert_eq!(resolved_config.reaper_exe_exists, true);
        assert_eq!(resolved_config.reaper_ini_exists, true);
        assert_eq!(resolved_config.portable, true);
        installer.install().await.unwrap();
        // Do basic assertions
        let executed = ExecutedTestCase {
            expected_dir,
            actual_dir,
        };
        executed.assert_binary_files_equal("ReaPack/registry.db");
        executed.assert_text_files_equal("reapack.ini");
        executed.assert_text_files_equal("reaper.ini");
        executed
    }
}

struct ExecutedTestCase {
    expected_dir: PathBuf,
    actual_dir: PathBuf,
}

impl ExecutedTestCase {
    fn assert_text_files_equal(&self, rel_path: &str) {
        assert_text_files_equal(
            &self.actual_dir.join(rel_path),
            &self.expected_dir.join(rel_path),
        );
    }

    fn assert_binary_files_equal(&self, rel_path: &str) {
        assert_binary_files_equal(
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
