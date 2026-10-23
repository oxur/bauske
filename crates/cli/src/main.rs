//! Thin executable entry point for the `relojban` command.

use std::{io, process::ExitCode};

fn main() -> ExitCode {
    bauske_cli::run(
        std::env::args_os(),
        io::stdout().lock(),
        io::stderr().lock(),
    )
}
