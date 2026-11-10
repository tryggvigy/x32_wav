# X32 WAV Extractor

Extract individual audio channels from Behringer X32 recordings made with the X-LIVE SD recording card.

X-LIVE stores all recorded channels together in multichannel WAV files. Long recordings span several numbered files within a recording session. This tool reads the session and produces one mono, 24-bit WAV file per selected channel. You can import these files into a DAW for editing and mixing.

I ported this project to Rust from an older Python script for X-LIVE recordings. The original Python code remains in this repository for reference. The Rust version runs without Python.

## Features

- Extract all channels or a single channel.
- Extract a complete session, a time range, or the audio between two markers.
- Combine audio across take files into continuous channel files.
- Show session information and marker positions.
- Change the session name shown on the mixer.

## Quick start

With Rust and Cargo installed, run this command from the repository directory:

```bash
cargo install --path .
```

The command installs `x32_wav` in Cargo's binary directory. This directory must be on your `PATH`.

Copy a recording session from the SD card to your computer. Keep its log and numbered WAV files together:

```text
recording/
├── SE_LOG.BIN
├── 00000001.WAV
├── 00000002.WAV
└── ...
```

Open a terminal in that recording directory:

```bash
cd /path/to/recording
```

Show the session information:

```bash
x32_wav getSessionInfo
```

Extract all channels:

```bash
x32_wav extractSession
```

The command creates a directory named `session_<session-id>` inside the recording directory. This directory contains `ch_1.wav`, `ch_2.wav`, and the remaining channels. Each file contains that channel's audio across the complete session.

The output directory must not already exist.

## Commands

Run all commands from the recording directory.

| Example | Result |
| --- | --- |
| `x32_wav getSessionInfo` | Show session information and marker indices. |
| `x32_wav extractSession` | Extract all channels for the complete session. |
| `x32_wav extractSessionTime 10 30` | Extract all channels from 10 seconds to 30 seconds. |
| `x32_wav extractSessionMarker 0 1` | Extract all channels between markers 0 and 1. |
| `x32_wav extractChannel 3` | Extract channel 3 for the complete session. |
| `x32_wav extractChannelTime 3 10 30` | Extract channel 3 from 10 seconds to 30 seconds. |
| `x32_wav extractChannelMarker 3 0 1` | Extract channel 3 between markers 0 and 1. |
| `x32_wav nameSession "Rehearsal"` | Change the session name in `SE_LOG.BIN`. |

Channel numbers start at 1. Marker indices start at 0. Time arguments use whole seconds. Time and marker selections include the start sample and exclude the end sample.

Extraction creates a new output directory whose name includes the selection and session ID. Single-channel extraction keeps the original channel number in the filename, such as `ch_3.wav`.

Session names accept up to 19 UTF-8 bytes without NUL characters.

Show all commands or the arguments for a specific command:

```bash
x32_wav --help
x32_wav extractChannelTime --help
```

## Format and limits

The X-LIVE card records 8, 16, or 32 channels in 32-bit PCM WAV files. This tool extracts the 24-bit audio from those files without changing the sample rate.

- A recording directory must contain `SE_LOG.BIN` and its numbered take files.
- Input filenames must match the uppercase names shown in the quick start on filesystems that distinguish letter case.
- Extraction rejects invalid channels, empty ranges, and ranges outside the session.
- Each output uses classic RIFF/WAV. Its size must fit the 32-bit RIFF fields. RF64 output is not supported.
- Extraction opens one take file at a time and processes audio in bounded blocks.
- An I/O error can leave an incomplete output directory. Errors return a nonzero exit status.

See the [Behringer X-LIVE guide](https://www.parts-express.com/pedocs/manuals/248-7000--behringer-xlive-manual.pdf) for the recording format and the card's operation.

## History and original Python script

This project started as a Rust port of the older `xlive_lib` Python code. Behringer's X-LIVE guide mentions downloadable Python scripts for separating recordings into individual WAV files.

The bundled [Python code](xlive_lib/) and [original tutorial](python_simple_tutorial.txt) remain available for reference. That code uses Python 2 syntax. The exact upstream source and individual authorship are not confirmed.

The Python code also creates X-LIVE sessions from individual channel files. The Rust CLI currently supports extraction, session information, and session naming.

## Development

The workspace contains three crates:

| Crate | Location | Responsibility |
| --- | --- | --- |
| `x32_wav` | `src/` | Parse CLI arguments, select operations, and print results. |
| `xlive-session` | `crates/xlive-session/` | Parse session metadata, resolve sample ranges, and extract channels. |
| `pcm-wav` | `crates/pcm-wav/` | Write mono 24-bit PCM WAV headers, samples, and padding. |

Dependencies flow from the CLI through the session crate to the WAV crate. The Rust library API is in `xlive-session`.

To run the CLI without installation, use `cargo run` with the repository's manifest path from the recording directory:

```bash
cargo run --manifest-path /path/to/x32_wav/Cargo.toml -- getSessionInfo
```

Run the tests from the repository directory:

```bash
cargo test --workspace
```

Run the formatting and lint checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The tests use synthetic recordings. They cover metadata, sample conversion, take boundaries, buffer boundaries, WAV lengths, and invalid input.
