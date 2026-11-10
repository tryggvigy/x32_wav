//! Define the existing command names and typed argument validation.

use clap::{value_parser, Arg, Command};

/// Build the CLI grammar without reading arguments or accessing session files.
///
/// Channel numbers are one-based. Marker indices are zero-based. Time endpoints
/// use whole seconds. Session-specific range checks run during extraction.
pub fn command() -> Command {
    Command::new("X-LIVE Audio Extractor")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Extracts audio from X-LIVE sessions")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("nameSession").about("Names the session").arg(
                Arg::new("name")
                    .help("Session name (up to 19 UTF-8 bytes)")
                    .required(true),
            ),
        )
        .subcommand(Command::new("getSessionInfo").about("Gets session info"))
        .subcommand(
            Command::new("extractSession").about("Extracts all audio channels of the session"),
        )
        .subcommand(
            Command::new("extractSessionTime")
                .about("Extracts all channels within a time span")
                .arg(seconds("start", "Start time in seconds"))
                .arg(seconds("end", "End time in seconds (exclusive)")),
        )
        .subcommand(
            Command::new("extractSessionMarker")
                .about("Extracts all channels between markers")
                .arg(marker("start_marker", "Start marker index (zero-based)"))
                .arg(marker(
                    "end_marker",
                    "Stop marker index (zero-based, audio endpoint excluded)",
                )),
        )
        .subcommand(
            Command::new("extractChannel")
                .about("Extracts a single channel")
                .arg(channel()),
        )
        .subcommand(
            Command::new("extractChannelTime")
                .about("Extracts a single channel within a time span")
                .arg(channel())
                .arg(seconds("start", "Start time in seconds"))
                .arg(seconds("end", "End time in seconds (exclusive)")),
        )
        .subcommand(
            Command::new("extractChannelMarker")
                .about("Extracts a single channel between markers")
                .arg(channel())
                .arg(marker("start_marker", "Start marker index (zero-based)"))
                .arg(marker(
                    "end_marker",
                    "Stop marker index (zero-based, audio endpoint excluded)",
                )),
        )
}

/// Define a required channel number in the recorder's inclusive range 1–32.
fn channel() -> Arg {
    Arg::new("channel")
        .help("Channel number (one-based)")
        .required(true)
        .value_parser(value_parser!(u32).range(1..=32))
}

/// Define a required, nonnegative whole-second endpoint.
fn seconds(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .help(help)
        .required(true)
        .value_parser(value_parser!(u64))
}

/// Define a required, zero-based marker index without a session-specific bound.
fn marker(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name)
        .help(help)
        .required(true)
        .value_parser(value_parser!(usize))
}

#[cfg(test)]
mod tests {
    //! Check numeric argument parsing before command dispatch.

    use super::*;

    #[test]
    /// Reject malformed arguments and preserve typed values for valid requests.
    fn validates_numbers_before_dispatch() {
        command().debug_assert();
        for args in [
            vec!["x32_wav", "extractChannel", "abc"],
            vec!["x32_wav", "extractChannel", "0"],
            vec!["x32_wav", "extractSessionTime", "-1", "2"],
            vec!["x32_wav", "extractSessionMarker", "0"],
        ] {
            assert!(command().try_get_matches_from(args).is_err());
        }
        let matches = command()
            .try_get_matches_from(["x32_wav", "extractChannelTime", "2", "1", "3"])
            .unwrap();
        let (_, args) = matches.subcommand().unwrap();
        assert_eq!(args.get_one::<u32>("channel"), Some(&2));
        assert_eq!(args.get_one::<u64>("end"), Some(&3));
    }
}
