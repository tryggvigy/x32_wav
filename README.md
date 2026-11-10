# X-LIVE Audio Extractor

## Rust workspace

The workspace contains three crates. Dependencies flow from the CLI through the session crate to the WAV crate.

| Crate | Location | Responsibility |
| --- | --- | --- |
| `x32_wav` | `src/` | Parse CLI arguments, select operations, and print results. |
| `xlive-session` | `crates/xlive-session/` | Parse session metadata, resolve sample ranges, and extract channels. |
| `pcm-wav` | `crates/pcm-wav/` | Write mono 24-bit PCM WAV headers, samples, and padding. |

The CLI separates arguments, command dispatch, and session information into individual files.
The session crate separates metadata, range planning, audio conversion, extraction, and session names.
The session and WAV crates do not depend on Clap or print messages.

The Rust library API is now in `xlive-session`, instead of `x32_wav::helpers`.

## How to run

### Rust

Run commands from the directory that contains `SE_LOG.BIN` and the numbered take files.

```bash
cargo run -- <sub_command> <args>
```

Examples:

```bash
cargo run -- extractChannel 1
cargo run -- getSessionInfo
cargo run -- extractSession
cargo run -- extractSessionTime 10 30
cargo run -- extractChannelMarker 1 0 1
cargo run -- nameSession "Rehearsal"
```

Run `cargo run -- --help` for all commands.
Run `cargo run -- extractChannel --help` for command arguments.

Channel numbers start at 1. Marker indices start at 0.
Time and marker selections include the start sample and exclude the end sample.
Session names accept up to 19 UTF-8 bytes without NUL characters.

Output folders contain the session ID from the log file.
Extraction rejects existing output folders and invalid selections.
Each extraction opens one take file at a time and processes audio in bounded blocks.
WAV headers contain the selected sample count. RIFF sizes must fit in 32 bits.

An I/O error can leave an incomplete output folder. Errors return a nonzero exit status.

## Development

Run the tests for all crates:

```bash
cargo test --workspace
```

Run the formatting and lint checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

The tests use synthetic recordings. They cover metadata, sample conversion, take boundaries, buffer boundaries, WAV lengths, and invalid input.

### Python

I'm using docker right no so I don't have to poison my system with python2.

1. Start a python interpreter in a docker container with the current directory mounted as a volume.

```bash
docker run --rm -it -v $(pwd):/app -w /app python:2 python
```

2. Follow the instructions in `python_simple_tutorial.txt` to run the python script.

Example:
```bash
>>> from xlive_lib import *
>>> extractChannels()
```