//! AegisNode CLI Crate (`aegisctl`)
//! Cung cấp giao diện dòng lệnh tương tác trực quan cho AegisNode Engine.

pub mod args;
pub mod commands;
pub mod exit_codes;
pub mod formatter;

use args::CliArgs;
use clap::Parser;
pub use exit_codes::exit_code_for_error;
pub use formatter::Formatter;

/// Hàm entrypoint thực thi CLI từ mảng tham số dòng lệnh
pub async fn run_cli_with_args(args: Vec<String>) -> i32 {
    let parsed_args = match CliArgs::try_parse_from(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return exit_codes::GENERIC_FAILURE;
        }
    };

    match commands::handle_command(parsed_args).await {
        Ok(()) => exit_codes::SUCCESS,
        Err(err) => {
            Formatter::print_error(&err.to_string());
            exit_code_for_error(&err)
        }
    }
}
