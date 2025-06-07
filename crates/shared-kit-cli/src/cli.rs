use crate::{
    config::Config,
    subcommand::new_command::{NewCommand, new_command_action},
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

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
    crate::helper::logger::init_logger();

    let cli = SharedKitCli::parse();
    let mut config =
        Config::from_path(cli.config).with_context(|| format!("Failed to load CLI config"))?;

    match &cli.command {
        Commands::New(args) => new_command_action(&mut config, args),
    }
}
