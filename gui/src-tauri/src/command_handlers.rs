use crate::app_handle::ReabootAppHandle;
use crate::worker::ReabootWorkerCommand;
use crate::ReabootAppState;
use reaboot_lib::api::{ReabootCommand, ReabootConfig, ReabootEvent};
use reaboot_lib::reaper_util;
use tauri::{Manager, State};

#[tauri::command]
pub fn reaboot_command(
    command: ReabootCommand,
    app_handle: tauri::AppHandle,
    state: State<ReabootAppState>,
) {
    let app_handle = ReabootAppHandle(app_handle);
    let result = match command {
        ReabootCommand::Configure { config } => configure(config, &app_handle, state),
        ReabootCommand::StartInstallation => install(state),
        ReabootCommand::CancelInstallation => {
            todo!()
        }
    };
    if let Err(e) = result {
        app_handle.emit_generic_error(e);
    }
}

fn configure(
    config: ReabootConfig,
    app_handle: &ReabootAppHandle,
    state: State<ReabootAppState>,
) -> anyhow::Result<()> {
    let mut current_config = state.config.lock().unwrap();
    *current_config = config;
    app_handle.emit_initial_events(&current_config)?;
    Ok(())
}

fn install(state: State<ReabootAppState>) -> anyhow::Result<()> {
    let current_config = state.config.lock().unwrap().clone();
    state
        .worker_command_sender
        .blocking_send(ReabootWorkerCommand::Install(current_config))?;
    Ok(())
}
