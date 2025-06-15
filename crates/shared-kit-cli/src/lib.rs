use shared_kit_common::log_error;

use crate::cli::run_cli;

mod helper;
pub mod config;
pub mod constant;
pub mod subcommand;

mod cli;
mod components;

pub fn shared_kit_cli() {
    if let Err(e) = run_cli() {
        log_error!("{}", e.to_string())
    }
}
