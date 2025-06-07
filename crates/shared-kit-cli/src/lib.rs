use crate::cli::run_cli;

#[macro_use]
mod helper;
pub mod config;
pub mod constant;
pub mod subcommand;

mod components;
mod utils;
mod cli;

pub fn shared_kit_cli() {
    if let Err(e) = run_cli() {
        error_msg!("{}", e.to_string())
    }
}
