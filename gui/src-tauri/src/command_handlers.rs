use crate::app_handle::ReabootAppHandle;
use crate::worker::ReabootWorkerCommand;
use crate::ReabootAppState;
use reaboot_core::api::{InstallerConfig, ReabootCommand};
use tauri::State;

#[tauri::command]
pub fn reaboot_command(
    command: ReabootCommand,
    app_handle: tauri::AppHandle,
    state: State<ReabootAppState>,
) {
    let app_handle = ReabootAppHandle(app_handle);
    let result = match command {
        ReabootCommand::ConfigureInstallation { config } => configure(config, state),
        ReabootCommand::StartInstallation => install(state),
        ReabootCommand::CancelInstallation => {
            todo!()
        }
    };
    if let Err(e) = result {
        app_handle.emit_generic_error(e);
    }
}

fn configure(config: InstallerConfig, state: State<ReabootAppState>) -> anyhow::Result<()> {
    let mut current_config = state.installer_config.lock().unwrap();
    *current_config = config;
    state.worker_command_sender.blocking_send(
        ReabootWorkerCommand::EmitInitialInstallationEvents(current_config.clone()),
    )?;
    Ok(())
}

fn install(state: State<ReabootAppState>) -> anyhow::Result<()> {
    let current_config = state.installer_config.lock().unwrap().clone();
    state
        .worker_command_sender
        .blocking_send(ReabootWorkerCommand::Install(current_config))?;
    Ok(())
}
