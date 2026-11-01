//! The `cargo-fallow` binary. The library crate holds the logic.

use std::process::ExitCode;

fn main() -> ExitCode {
    cargo_fallow::run(std::env::args_os().collect())
}
