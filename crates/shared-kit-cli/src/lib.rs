use shared_kit_common::{log_debug, output};

use crate::cli::run_cli;

pub mod config;
pub mod constant;
mod helper;
pub mod subcommand;

mod cli;
mod components;

pub fn shared_kit_cli() {
    if let Err(e) = run_cli() {
        log_debug!("cli run error: \n {}", e.to_string());
        output!(error: "Error: {}", e.to_string());
    }
}
