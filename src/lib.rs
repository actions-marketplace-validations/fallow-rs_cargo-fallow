//! `cargo fallow`: codebase analysis for Rust workspaces.
//!
//! The `cargo-fallow` binary is a thin wrapper around [`run`]. The library
//! holds every module, so the `missing_docs` lint checks the public API.
//! The public API is [`run`] only. The modules are internal.

// Only the workspace loader spawns a process. See `.clippy.toml`.
#![cfg_attr(not(test), deny(clippy::disallowed_methods))]

mod cli;
mod error;
mod gates;
mod render;
mod report;
mod workspace;

use std::ffi::OsString;
use std::process::ExitCode;

use fallow_report_protocol::exit_code::{EXIT_GATE_FAILED, EXIT_INVALID_INPUT, EXIT_OK};

use crate::cli::{Options, Parsed};
use crate::error::CliError;

/// Runs the CLI with the process arguments and returns the exit code.
///
/// `args` starts with the binary name. Cargo runs the binary as
/// `cargo-fallow fallow <args>`, and a direct call runs it as
/// `cargo-fallow <args>`. Both forms give the same result.
pub fn run(args: Vec<OsString>) -> ExitCode {
    let args = cli::strip_cargo_subcommand(args);
    let code = match cli::parse(&args) {
        Parsed::Run(options) => run_check(&options),
        Parsed::Exit(code) => code,
    };
    ExitCode::from(code)
}

/// Runs `check` and returns the process exit code.
fn run_check(options: &Options) -> u8 {
    match check(options) {
        Ok(code) => code,
        Err(err) => {
            err.emit(options.format);
            EXIT_INVALID_INPUT
        }
    }
}

fn check(options: &Options) -> Result<u8, CliError> {
    let progress = render::Progress::new(options.quiet);
    progress.step("Reading cargo metadata");
    let workspace = workspace::load(options.manifest_path.as_deref())?;
    progress.step("Parsing target root files");
    let outcomes = gates::evaluate(&workspace, options);
    let fails = outcomes.fails_run();
    let report = report::CheckReport::new(workspace, outcomes);
    render::write_report(&report, options.format).map_err(|err| CliError::output(&err))?;
    Ok(if fails { EXIT_GATE_FAILED } else { EXIT_OK })
}
