//! Command-line interface for X-LIVE session inspection and audio extraction.
//!
//! Commands operate on `SE_LOG.BIN` and numbered take files in the current
//! directory. [`args`] defines the command grammar, [`command`] dispatches parsed
//! requests, and [`info`] prints metadata. The `xlive_session` and `pcm_wav`
//! crates contain the file-format and extraction logic.
//!
//! Clap reports argument errors before session access. Operational errors print
//! to standard error and produce a failing process exit status.

mod args;
mod command;
mod info;

use std::process::ExitCode;

/// Parse arguments, dispatch the requested operation, and report its exit status.
///
/// Clap exits directly for help, version requests, or invalid arguments.
fn main() -> ExitCode {
    match command::run(args::command().get_matches()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
