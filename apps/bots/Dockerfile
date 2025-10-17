# syntax=docker/dockerfile:1

# Stage 1: Builder
FROM rust:1.83-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build
RUN cargo build --release --bin market_maker

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/market_maker /usr/local/bin/market_maker

# Copy .env.defaults
COPY .env.defaults .env.defaults

# Run
CMD ["market_maker"]
