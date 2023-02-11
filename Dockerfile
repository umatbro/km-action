FROM rust:slim-buster

RUN apt update && apt install -y libssl-dev pkg-config

COPY src /src
COPY Cargo.toml /
COPY Cargo.lock /

RUN cargo build --release

ENTRYPOINT ["/target/release/be-keto-mojo-gh-bot-action"]