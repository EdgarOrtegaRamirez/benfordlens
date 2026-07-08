# BenfordLens Docker Image

FROM rust:1.96-slim AS builder

WORKDIR /root/benfordlens
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /root/benfordlens/target/release/benfordlens /usr/local/bin/

RUN benfordlens --version

ENTRYPOINT ["benfordlens"]
