#!/bin/bash

echo "🚀 Setting up development environment..."

# Configure Cargo for container (reduce memory usage)
echo "⚙️  Configuring Cargo for container environment..."
mkdir -p /workspace/.cargo
cat > /workspace/.cargo/config.toml << 'EOF'
[build]
# Reduce parallel jobs to save memory in container
jobs = 2

[profile.dev]
# Reduce debuginfo to save memory during linking
debug = 1
incremental = true

[profile.test]
# Reduce debuginfo in tests
debug = 1

# Use mold linker for faster, less memory-intensive linking
[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF

# Wait for docker daemon to be ready
echo "⏳ Waiting for Docker daemon..."
max_attempts=30
attempt=0
until docker info > /dev/null 2>&1 || [ $attempt -eq $max_attempts ]; do
  echo "Waiting for Docker daemon to start..."
  sleep 2
  attempt=$((attempt + 1))
done

if [ $attempt -eq $max_attempts ]; then
  echo "❌ Docker daemon failed to start"
  exit 1
fi

echo "✅ Docker daemon is ready!"

# Start databases using docker compose
echo "🗄️  Starting databases (PostgreSQL and ClickHouse)..."
cd /workspace
just db-run

# Wait for databases to be ready
echo "⏳ Waiting for databases to be ready..."
until pg_isready -h localhost -p 5432 -U postgres; do
  echo "Waiting for PostgreSQL..."
  sleep 2
done

until docker exec exchange-clickhouse clickhouse-client --query "SELECT 1" > /dev/null 2>&1; do
  echo "Waiting for ClickHouse..."
  sleep 2
done

echo "✅ Databases are ready!"

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
cd /workspace/apps/frontend && bun install

# Run database migrations
echo "🗄️  Running database migrations..."
cd /workspace
just db-setup || echo "⚠️  Database setup failed. You may need to run 'just db-setup' manually."

# Build backend to check everything works
echo "🔨 Building backend..."
cd /workspace/apps/backend && cargo build || echo "⚠️  Backend build failed. You may need to fix compilation errors."

echo ""
echo "✅ Development environment is ready!"
echo ""
echo "📚 Available commands:"
echo "  just backend   - Run the backend server"
echo "  just frontend  - Run the frontend dev server"
echo "  just db-setup  - Set up databases"
echo "  just db-reset  - Reset databases"
echo "  just test      - Run tests"
echo "  just openapi   - Generate OpenAPI schema"
echo ""
echo "💡 Databases are running via Docker-in-Docker:"
echo "  docker compose ps    - Check database status"
echo "  docker compose logs  - View database logs"
echo ""
