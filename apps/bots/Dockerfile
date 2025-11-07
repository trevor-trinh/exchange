
# Stage 1: Builder
FROM rust:1.90-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# get environment variables from Railway
ARG EXCHANGE_URL

# set environment variables from Railway
ENV EXCHANGE_URL=$EXCHANGE_URL

# Build
RUN cargo build --release --bin bots

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and config
COPY --from=builder /app/target/release/bots /usr/local/bin/bots
COPY --from=builder /app/apps/bots/config.toml /app/config.toml

# Set config path
ENV BOTS_CONFIG=/app/config.toml

# Run
CMD ["bots"]
