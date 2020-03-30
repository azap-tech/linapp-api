# Build

## install rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You will need to add `~/.cargo/bin` to you `$PATH`
You also need some tools

```
cargo install systemfd
cargo install cargo-watch
cargo install diesel_cli --no-default-features --features "postgres"
```

## Build

```bash
# fast build to check for errors
cargo check
# build
cargo build
# build for production
cargo build --release
# build and watch
systemfd --no-pid -s http::8080 -- cargo watch -x "run --release"
```

## Run

Run the `./tools/run-env.sh` script to start postgres docker and run migration.
`cargo run` or `systemfd --no-pid -s http::8080 -- cargo watch -x "run --release"` to rebuild on files change.

# Test

```bash
cd tools/test-client
# setup python virtual env
python -m venv env
source env/bin/activate
pip install -r requirements.txt
# run test script
python main.py
# You can also load main.py and call api functions directly
python
>>> from main import *
>>> create_doctor("name", "phone", 0)
```
