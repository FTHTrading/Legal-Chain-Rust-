# Dockerfile — Multi-stage build for LEGAL-CHAIN node
#
# Build:   docker build -t legal-chain-node .
# Run:     docker run --rm -p 9944:9944 -p 30333:30333 legal-chain-node --dev

# ── Stage 1: Builder ────────────────────────────────────────────────
FROM rust:1.88-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    protobuf-compiler \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .
RUN cargo build --release

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -U -s /bin/sh -d /legal-chain legal-chain
COPY --from=builder /build/target/release/legal-chain-node /usr/local/bin/

USER legal-chain
EXPOSE 9944 30333

VOLUME ["/data"]

ENTRYPOINT ["legal-chain-node"]
CMD ["--dev"]
