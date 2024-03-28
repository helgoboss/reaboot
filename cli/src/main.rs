use crate::commands::install::{install, InstallArgs};
use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use indicatif::TermLike;
use reaboot_core::installer::InstallerListener;
use std::fmt::Display;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = init_tracing();
    let app = App::parse();
    match app.command {
        Command::Install(args) => install(args).await,
    }
}

fn init_tracing() -> anyhow::Result<()> {
    let env_var = std::env::var("REABOOT_LOG")?;
    let env_filter = EnvFilter::new(env_var);
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        // .compact()
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;
    Ok(())
}

#[derive(Debug, Parser)]
#[clap(version)]
pub struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Install(InstallArgs),
}
