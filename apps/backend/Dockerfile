# syntax=docker/dockerfile:1

# Stage 1: Builder
FROM rust:1.83-slim AS builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src
COPY scripts ./scripts

# Build
RUN cargo build --release --bin backend

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# Copy .env.defaults (not .env - that's local only)
COPY .env.defaults .env.defaults

# Expose port
EXPOSE 8888

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8888/api/health || exit 1

# Run
CMD ["backend"]
