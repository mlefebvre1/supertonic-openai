FROM rust:slim-trixie

RUN apt-get update && apt-get install -y build-essential libssl-dev pkg-config

WORKDIR /app
COPY . .

RUN cargo build --release

ENTRYPOINT ["cargo", "run", "--release"]