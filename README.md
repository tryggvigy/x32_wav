## How to run

### Rust
```bash
cargo run -- <sub_command> <args>
```

Examples:
```bash
cargo run -- extractChannel 1
cargo run -- getSessionInfo
cargo run -- extractChannels
```

Run `cargo run -- --help` for more information.
Help also works inside subcommands, e.g. `cargo run -- extractChannel --help`.


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