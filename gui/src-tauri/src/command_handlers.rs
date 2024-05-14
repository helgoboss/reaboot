use anyhow::Context;
use std::path::PathBuf;

use tauri::State;

use reaboot_core::api::InstallerConfig;
use reaboot_core::reaboot_util::resolve_config;
use reaboot_core::reaper_util;

use crate::api::ReabootCommand;
use crate::app_handle::ReabootAppHandle;
use crate::config_util::resolve_config_and_send_events;
use crate::worker::ReabootWorkerCommand;
use crate::ReabootAppState;

#[tauri::command]
pub async fn get_reaper_eula() -> Result<String, String> {
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
    let app_handle = ReabootAppHandle::new(app_handle);
    let result = match command {
        ReabootCommand::ConfigureInstallation { config } => {
            configure(app_handle, config, state).await
        }
        ReabootCommand::StartInstallation => install(state).await,
        ReabootCommand::StartReaper => start_reaper(state).await,
        ReabootCommand::StartReaperInstaller { path } => start_reaper_installer(path),
        ReabootCommand::Confirm { answer } => confirm(state, answer),
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
    resolve_config_and_send_events(config.clone(), app_handle).await?;
    // Only write config if that was successful
    *state.installer_config.lock().unwrap() = config;
    Ok(())
}

async fn install(state: State<'_, ReabootAppState>) -> anyhow::Result<()> {
    let command = ReabootWorkerCommand::Install {
        config: state.extract_installer_config(),
        temp_dir_for_reaper_download: state.temp_dir_for_reaper_download.path().to_path_buf(),
    };
    state.worker_command_sender.send(command).await?;
    Ok(())
}

async fn start_reaper(state: State<'_, ReabootAppState>) -> anyhow::Result<()> {
    let resolved_config = resolve_config(state.extract_installer_config()).await?;
    reaper_util::start_reaper(&resolved_config.reaper_exe)?;
    Ok(())
}

fn start_reaper_installer(path: PathBuf) -> anyhow::Result<()> {
    reaper_util::start_reaper_installer(&path)?;
    Ok(())
}

fn confirm(state: State<'_, ReabootAppState>, answer: bool) -> anyhow::Result<()> {
    state.interaction_sender.send(answer)?;
    Ok(())
}
