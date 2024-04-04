use crate::worker::ReabootWorkerCommand;
use reaboot_core::api::InstallerConfig;
use reaboot_core::recipe::{find_and_parse_public_recipe, Recipe};
use std::sync::Mutex;
use tempdir::TempDir;

pub struct ReabootAppState {
    /// Non-enhanced config from client (doesn't contain package URLs from recipe).
    pub installer_config: Mutex<InstallerConfig>,
    pub worker_command_sender: tauri::async_runtime::Sender<ReabootWorkerCommand>,
    pub temp_dir_for_reaper_download: TempDir,
    pub recipe_id: Option<String>,
    pub recipe: Mutex<Option<Recipe>>,
}

impl ReabootAppState {
    pub fn extract_installer_config(&self) -> InstallerConfig {
        self.installer_config.lock().unwrap().clone()
    }

    pub fn extract_recipe(&self) -> Option<Recipe> {
        self.recipe.lock().unwrap().clone()
    }

    /// Returns `true` if a recipe was set.
    pub async fn fill_recipe_if_necessary(&self) -> bool {
        let Some(recipe_id) = self.recipe_id.as_ref() else {
            return false;
        };
        let recipe = find_and_parse_public_recipe(recipe_id).await;
        if let Some(r) = recipe {
            *self.recipe.lock().unwrap() = Some(r);
            true
        } else {
            false
        }
    }
}
