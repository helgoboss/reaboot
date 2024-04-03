use crate::api::{ReabootCommand, ReabootEvent};
use crate::app_handle::ReabootAppHandle;
use crate::worker::ReabootWorkerCommand;
use crate::ReabootAppState;
use anyhow::Context;
use reaboot_core::api::InstallerConfig;
use reaboot_core::reaboot_util::resolve_config;
use reaboot_core::reaper_platform::ReaperPlatform;
use reaboot_core::{reaboot_util, reaper_util};
use reaboot_reapack::model::PackageUrl;
use tauri::State;

#[tauri::command]
pub async fn reaper_eula() -> Result<String, String> {
    reaper_util::get_reaper_eula()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reaboot_command(
    command: ReabootCommand,
    app_handle: tauri::AppHandle,
    state: State<'_, ReabootAppState>,
) -> Result<(), String> {
    let app_handle = ReabootAppHandle(app_handle);
    let result = match command {
        ReabootCommand::ConfigureInstallation { config } => {
            configure(app_handle, config, state).await
        }
        ReabootCommand::StartInstallation => install(state).await,
        ReabootCommand::StartReaper => start_reaper(state),
        ReabootCommand::CancelInstallation => {
            todo!()
        }
    };
    result.map_err(|r| r.to_string())?;
    Ok(())
}

async fn configure(
    app_handle: ReabootAppHandle,
    config: InstallerConfig,
    state: State<'_, ReabootAppState>,
) -> anyhow::Result<()> {
    // Resolve config
    let resolved_config = resolve_config(config.clone()).await?;
    // Only write config if successfully resolved
    *state.installer_config.lock().unwrap() = config;
    // Send derived events
    let installation_stage =
        reaboot_util::determine_initial_installation_stage(&resolved_config).await?;
    let backend_info = reaboot_util::collect_backend_info();
    app_handle.emit_reaboot_event(ReabootEvent::BackendInfoChanged { info: backend_info });
    app_handle.emit_reaboot_event(ReabootEvent::ConfigResolved {
        config: resolved_config,
    });
    app_handle.emit_reaboot_event(ReabootEvent::installation_stage_changed(installation_stage));
    Ok(())
}

async fn install(state: State<'_, ReabootAppState>) -> anyhow::Result<()> {
    let current_config = state.installer_config.lock().unwrap().clone();
    state
        .worker_command_sender
        .send(ReabootWorkerCommand::Install(current_config))
        .await?;
    Ok(())
}

fn start_reaper(state: State<ReabootAppState>) -> anyhow::Result<()> {
    let current_config = state.installer_config.lock().unwrap();
    let platform = current_config
        .custom_platform
        .or(ReaperPlatform::BUILD)
        .context("couldn't identify REAPER platform")?;
    reaper_util::start_reaper(current_config.custom_reaper_resource_dir.as_ref(), platform)?;
    Ok(())
}
