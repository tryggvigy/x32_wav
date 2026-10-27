//! Translate parsed CLI commands into session operations and output folder names.

use clap::ArgMatches;
use std::io;
use std::time::Instant;
use xlive_session::{Channels, Interval, Session};

/// Run matches produced by [`crate::args::command`] in the current directory.
///
/// Naming and information commands print their results directly. Extraction
/// commands select channels and an interval, derive the existing folder naming
/// convention from the command, and print elapsed time after completion.
///
/// # Errors
///
/// Returns errors from opening the session, renaming it, or extracting audio.
/// The caller reports the error and chooses the process exit status.
///
/// # Panics
///
/// Panics if `matches` does not contain the required subcommand or its expected
/// typed arguments. The CLI grammar prevents these cases.
pub fn run(matches: ArgMatches) -> io::Result<()> {
    let (command, args) = matches.subcommand().expect("Clap requires a subcommand");
    let session = Session::open(".")?;
    match command {
        "nameSession" => {
            session.rename(args.get_one::<String>("name").expect("required name"))?;
            println!("Session named successfully!");
        }
        "getSessionInfo" => crate::info::print(session.metadata()),
        _ => {
            let start = Instant::now();
            let (channels, channel_prefix) = if command.starts_with("extractChannel") {
                let channel = *args.get_one::<u32>("channel").expect("required channel");
                (Channels::One(channel), Some(channel))
            } else {
                (Channels::All, None)
            };
            let (interval, suffix) = if command.ends_with("Time") {
                let start = *args.get_one::<u64>("start").expect("required start");
                let end = *args.get_one::<u64>("end").expect("required end");
                (
                    Interval::Seconds(start..end),
                    format!("_time_{start}_{end}"),
                )
            } else if command.ends_with("Marker") {
                let start = *args
                    .get_one::<usize>("start_marker")
                    .expect("required start marker");
                let end = *args
                    .get_one::<usize>("end_marker")
                    .expect("required end marker");
                (
                    Interval::Markers(start..end),
                    format!("_marker_{start}_{end}"),
                )
            } else {
                (Interval::Full, String::new())
            };
            let prefix = if channel_prefix.is_some() {
                "channel"
            } else {
                "session"
            };
            let channel_suffix = channel_prefix
                .map(|channel| format!("_{channel}"))
                .unwrap_or_default();
            let folder = format!(
                "{prefix}{suffix}{channel_suffix}_{:08x}",
                session.metadata().session_id()
            );
            println!("Unpacking audio data into {folder}");
            session.extract(channels, interval, &folder)?;
            println!("process completed in={}sec", start.elapsed().as_secs());
        }
    }
    Ok(())
}
