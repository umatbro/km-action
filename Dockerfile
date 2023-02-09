FROM rust:slim-buster

COPY src /src
COPY Cargo.toml /
COPY Cargo.lock /

RUN cargo build --release

ENTRYPOINT ["/target/release/be-keto-mojo-gh-bot-action"]