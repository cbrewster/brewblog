FROM rust:1.43-slim

RUN apt-get update

RUN apt-get -y install clang

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY assets assets
COPY src src

RUN cargo install --path . && rm -rf target
