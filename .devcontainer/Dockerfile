# Use the latest version of Ubuntu as the base for our environment.
FROM ubuntu:24.04

# Avoids prompts from package installers.
ARG DEBIAN_FRONTEND=noninteractive

# Update packages and install build tools needed for compiling Rust crates.
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    libssl-dev \
    pkg-config \
    curl \
    git \
    docker.io \
    wget \
    ca-certificates \
    gnupg \
    lsb-release \
    unzip \
    postgresql-client \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Set the Rustup and Cargo home directories (will be accessible to all users).
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# Install rustup and the default stable Rust toolchain.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.90 \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# Install mold linker for faster, less memory-intensive linking
RUN curl -fsSL https://github.com/rui314/mold/releases/download/v2.4.0/mold-2.4.0-aarch64-linux.tar.gz | tar -xz -C /usr/local --strip-components=1

# Install Rust tools (separately to avoid build failures affecting each other)
RUN cargo install sqlx-cli --no-default-features --features rustls,postgres
RUN cargo install just

# Install Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Bun
RUN curl -fsSL https://bun.sh/install | bash \
    && mv /root/.bun /usr/local/bun \
    && ln -s /usr/local/bun/bin/bun /usr/local/bin/bun

# Install ClickHouse CLI
RUN curl https://clickhouse.com/ | sh \
    && mv ./clickhouse /usr/local/bin/clickhouse \
    && chmod +x /usr/local/bin/clickhouse

# Install pgcli (better PostgreSQL CLI)
RUN pip3 install --break-system-packages pgcli

# Create directories for volume mounts and make them accessible to all users
RUN mkdir -p /var/lib/docker /usr/local/cargo/registry /usr/local/rustup \
    && chmod -R 777 /usr/local/cargo /usr/local/rustup

# Set the working directory inside the container.
WORKDIR /workspace

# Keep the container alive for development.
CMD ["sleep", "infinity"]
