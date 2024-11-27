mod extract;
mod helpers;

use clap::{Arg, Command};

fn main() {
    let matches = Command::new("X-LIVE Audio Extractor")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Extracts audio from X-LIVE sessions")
        .subcommand(
            Command::new("nameSession").about("Names the session").arg(
                Arg::new("name")
                    .help("The name of the session")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(Command::new("getSessionInfo").about("Gets session info"))
        .subcommand(
            Command::new("extractSession").about("Extracts all audio channels of the session"),
        )
        .subcommand(
            Command::new("extractSessionTime")
                .about("Extracts all audio channels of the session within a time span")
                .arg(
                    Arg::new("start")
                        .help("Start time in seconds")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("end")
                        .help("End time in seconds")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("extractSessionMarker")
                .about("Extracts all audio channels of the session between markers")
                .arg(
                    Arg::new("start_marker")
                        .help("Start marker index")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("end_marker")
                        .help("End marker index")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("extractChannel")
                .about("Extracts a single channel of the session")
                .arg(
                    Arg::new("channel")
                        .help("Channel number")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("extractChannelTime")
                .about("Extracts a single channel of the session within a time span")
                .arg(
                    Arg::new("channel")
                        .help("Channel number")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("start")
                        .help("Start time in seconds")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("end")
                        .help("End time in seconds")
                        .required(true)
                        .index(3),
                ),
        )
        .subcommand(
            Command::new("extractChannelMarker")
                .about("Extracts a single channel of the session between markers")
                .arg(
                    Arg::new("channel")
                        .help("Channel number")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("start_marker")
                        .help("Start marker index")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("end_marker")
                        .help("End marker index")
                        .required(true)
                        .index(3),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("nameSession", sub_m)) => {
            let name = sub_m.get_one::<String>("name").unwrap();
            if let Err(e) = extract::name_session(name) {
                eprintln!("Error naming session: {}", e);
            }
        }
        Some(("getSessionInfo", _)) => {
            if let Err(e) = extract::get_session_info() {
                eprintln!("Error getting session info: {}", e);
            }
        }
        Some(("extractSession", _)) => {
            if let Err(e) = extract::extract_session() {
                eprintln!("Error extracting session: {}", e);
            }
        }
        Some(("extractSessionTime", sub_m)) => {
            let start = sub_m.get_one::<String>("start").unwrap().parse().unwrap();
            let end = sub_m.get_one::<String>("end").unwrap().parse().unwrap();
            if let Err(e) = extract::extract_session_time(start, end) {
                eprintln!("Error extracting session time: {}", e);
            }
        }
        Some(("extractSessionMarker", sub_m)) => {
            let start_marker = sub_m
                .get_one::<String>("start_marker")
                .unwrap()
                .parse()
                .unwrap();
            let end_marker = sub_m
                .get_one::<String>("end_marker")
                .unwrap()
                .parse()
                .unwrap();
            if let Err(e) = extract::extract_session_marker(start_marker, end_marker) {
                eprintln!("Error extracting session marker: {}", e);
            }
        }
        Some(("extractChannel", sub_m)) => {
            let channel = sub_m.get_one::<String>("channel").unwrap().parse().unwrap();
            if let Err(e) = extract::extract_channel(channel) {
                eprintln!("Error extracting channel: {}", e);
            }
        }
        Some(("extractChannelTime", sub_m)) => {
            let channel = sub_m.get_one::<String>("channel").unwrap().parse().unwrap();
            let start = sub_m.get_one::<String>("start").unwrap().parse().unwrap();
            let end = sub_m.get_one::<String>("end").unwrap().parse().unwrap();
            if let Err(e) = extract::extract_channel_time(channel, start, end) {
                eprintln!("Error extracting channel time: {}", e);
            }
        }
        Some(("extractChannelMarker", sub_m)) => {
            let channel = sub_m.get_one::<String>("channel").unwrap().parse().unwrap();
            let start_marker = sub_m
                .get_one::<String>("start_marker")
                .unwrap()
                .parse()
                .unwrap();
            let end_marker = sub_m
                .get_one::<String>("end_marker")
                .unwrap()
                .parse()
                .unwrap();
            if let Err(e) = extract::extract_channel_marker(channel, start_marker, end_marker) {
                eprintln!("Error extracting channel marker: {}", e);
            }
        }
        _ => eprintln!("Unknown command"),
    }
}
