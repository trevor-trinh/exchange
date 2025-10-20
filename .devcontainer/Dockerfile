# Use the latest version of Ubuntu as the base for our environment.
FROM ubuntu:latest

# Avoids prompts from package installers.
ARG DEBIAN_FRONTEND=noninteractive

# Update packages and install build tools needed for compiling Rust crates.
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    curl \
    git \
    docker.io \
    && rm -rf /var/lib/apt/lists/*

# Set the Rustup and Cargo home directories.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Install rustup and the default stable Rust toolchain.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
RUN cargo install just

# Create directories for volume mounts
RUN mkdir -p /var/lib/docker /usr/local/cargo/registry /usr/local/rustup


# clickhouse cli
# pgcli
# node
# bun


# Set the working directory inside the container.
WORKDIR /app

# Keep the container alive for development.
CMD ["sleep", "infinity"]
