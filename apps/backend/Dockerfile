# Stage 1: Builder
FROM rust:1.90-bookworm AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (needed for workspace dependencies)
COPY apps/backend ./apps/backend
COPY apps/bots ./apps/bots
COPY packages/sdk-rust ./packages/sdk-rust
COPY packages/test-utils ./packages/test-utils
COPY packages/schema-generator ./packages/schema-generator

# get environment variables from Railway
ARG PG_URL
ARG CH_URL

# set environment variables from Railway
ENV PG_URL=$PG_URL
ENV CH_URL=$CH_URL

# Build from workspace root
RUN cargo build --release --bin backend --bin init_exchange

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
COPY --from=builder /app/target/release/init_exchange /usr/local/bin/init_exchange

# Copy config to the path expected by CARGO_MANIFEST_DIR (compiled into binary)
COPY --from=builder /app/apps/backend/config.toml /app/apps/backend/config.toml

ENV PORT=8888
EXPOSE 8888

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8888/api/health || exit 1

# Run backend server
CMD ["backend"]
