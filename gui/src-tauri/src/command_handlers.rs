use tauri::State;

use reaboot_core::api::InstallerConfig;
use reaboot_core::reaboot_util::resolve_config;
use reaboot_core::{reaboot_util, reaper_util};

use crate::api::{ReabootCommand, ReabootEvent};
use crate::app_handle::ReabootAppHandle;
use crate::worker::ReabootWorkerCommand;
use crate::ReabootAppState;

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
        ReabootCommand::StartReaper => start_reaper(state).await,
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

async fn start_reaper(state: State<'_, ReabootAppState>) -> anyhow::Result<()> {
    let config = state.installer_config.lock().unwrap().clone();
    let resolved_config = resolve_config(config).await?;
    reaper_util::start_reaper(&resolved_config.reaper_exe)?;
    Ok(())
}
