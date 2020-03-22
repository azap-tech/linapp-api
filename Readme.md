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
# fast build to check for erros
cargo check
# build
cargo build
# build for production
cargo build --release
# build and watch
systemfd --no-pid -s http::8080 -- cargo watch -x run
```

## Run

Run the `./tools/run-env.sh` script to start postgres docker and run migration.
`cargo run` or `systemfd --no-pid -s http::8080 -- cargo watch -x run` to rebuild on files change.

# Test

```bash
# setup python virtual env
python -m venv env
source env/bin/activate
pip install -r requirement.txt
# run test script
python tools/test.py
```

# Deploy

## Setup heroku

```bash
heroku login
git remote add dev https://git.heroku.com/azap-dev.git
git remote add heroku https://git.heroku.com/azap-prod.git
```

```

## Deploy to heroku

`git push heroku master` Or `git push dev master`
```

Load scripts
python3 load_barber.py stores/barber_avenue.json 