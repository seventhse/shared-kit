use std::path::PathBuf;

use crate::{
    config::Config,
    constant::DEFAULT_CONFIG_DIR,
    subcommand::new_command::{NewCommand, new_command_action},
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use shared_kit_common::tracing::Level;

#[derive(Parser)]
#[command(
    version = "0.1.0",
    about = "Unified CLI toolkit for managing projects and developer utilities",
    long_about = Some(
        "A flexible and extensible command-line toolkit for initializing projects, managing shared configs, and running developer workflows.\n\n\
         Supports monorepos, language-specific templates, utility automation, and custom developer operations."
    ),
    propagate_version = true
)]
struct SharedKitCli {
    /// Custom config file path (default: $HOME/.config/shared-kit-cli.toml, can be overridden by subcommand config)
    #[arg(short = 'c', long = "config", value_name = "CONFIG")]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New(NewCommand),
}

pub fn run_cli() -> Result<()> {
    let user_home_dir = shared_kit_common::dirs::home_dir().unwrap();
    let _guard = shared_kit_common::logger::init_logger(
        Some(PathBuf::from(format!(
            "{}/{}/logs",
            user_home_dir.to_string_lossy(),
            DEFAULT_CONFIG_DIR
        ))),
        Level::INFO,
        Level::DEBUG,
    );

    let cli = SharedKitCli::parse();
    let mut config =
        Config::from_path(cli.config).with_context(|| format!("Failed to load CLI config"))?;

    match &cli.command {
        Commands::New(args) => new_command_action(&mut config, args),
    }
}
