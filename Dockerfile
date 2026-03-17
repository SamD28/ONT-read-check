# Build stage
FROM rust:slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    procps && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/read_check /usr/local/bin/read_check

ENTRYPOINT ["read_check"]
