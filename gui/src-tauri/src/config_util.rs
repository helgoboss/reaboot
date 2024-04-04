use crate::api::ReabootEvent;
use crate::app_handle::ReabootAppHandle;
use reaboot_core::api::InstallerConfig;
use reaboot_core::reaboot_util;
use reaboot_core::reaboot_util::resolve_config;
use reaboot_core::recipe::Recipe;

pub async fn resolve_config_and_send_events(
    config: InstallerConfig,
    recipe_id: Option<String>,
    recipe: Option<Recipe>,
    app_handle: ReabootAppHandle,
) -> anyhow::Result<()> {
    // Resolve enhanced config
    let resolved_config = resolve_config(config, recipe).await?;
    // Send derived events
    let installation_stage =
        reaboot_util::determine_initial_installation_stage(&resolved_config).await?;
    let backend_info = reaboot_util::collect_backend_info();
    // We keep it simple and send basically everything on "configure".
    if let Some(recipe_id) = recipe_id {
        app_handle.emit_reaboot_event(ReabootEvent::RecipeIdDetected { recipe_id });
    }
    app_handle.emit_reaboot_event(ReabootEvent::BackendInfoChanged { info: backend_info });
    app_handle.emit_reaboot_event(ReabootEvent::ConfigResolved {
        config: resolved_config,
    });
    app_handle.emit_reaboot_event(ReabootEvent::installation_stage_changed(installation_stage));
    Ok(())
}
