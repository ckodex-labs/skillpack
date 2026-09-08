# syntax=docker/dockerfile:1
# SkillPack Server — multi-stage build
# Targets: skillpack-server (gRPC + HTTP) + skillpack CLI

# --------------------------
# Stage 1: Build
# --------------------------
FROM rust:1.85-bookworm AS builder

WORKDIR /build

# Install system deps (duckdb bundled needs build tools)
RUN apt-get update && apt-get install -y \
    cmake \
    ninja-build \
    python3 \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache layer: copy workspace manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/
# Dummy main.rs files so cargo can build deps layer
RUN mkdir -p crates/skillpack-api/src/bin && \
    mkdir -p crates/skillpack-adapters/src/cli && \
    echo "fn main() {}" > crates/skillpack-api/src/bin/server.rs && \
    echo "fn main() {}" > crates/skillpack-adapters/src/cli/main.rs && \
    for d in crates/*/src; do touch "$d/lib.rs" 2>/dev/null || true; done

# Build dependencies (cached layer)
RUN cargo build --release --bin skillpack-server --bin skillpack 2>/dev/null || true

# Now copy real source
COPY . .

# Touch source files to force rebuild
RUN find crates -name "*.rs" -exec touch {} \;

# Build release binaries
RUN cargo build --release --bin skillpack-server --bin skillpack

# --------------------------
# Stage 2: Runtime
# --------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r skillpack && useradd -r -g skillpack skillpack

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/skillpack-server /usr/local/bin/skillpack-server
COPY --from=builder /build/target/release/skillpack /usr/local/bin/skillpack

# Data directory for file-backed DuckDB
RUN mkdir -p /data && chown skillpack:skillpack /data

USER skillpack

# Default env
ENV SKILLPACK_PORT=50051
ENV SKILLPACK_HTTP_PORT=50052
ENV SKILLPACK_DB_PATH=/data/skillpack.db

EXPOSE 50051 50052

CMD ["skillpack-server"]
