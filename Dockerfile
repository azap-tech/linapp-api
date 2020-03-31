FROM rust:latest as builder
WORKDIR /azap-api
#COPY . .
#RUN cargo install --path .
RUN cargo install systemfd
RUN cargo install cargo-watch
RUN cargo install diesel_cli --no-default-features --features "postgres"
CMD cargo run --release
