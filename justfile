export DATABASE_URL := "postgresql://postgres:password@localhost:5432/exchange"

default:
  just --list

backend:
  cd apps/backend && cargo run --release

frontend:
  cd apps/frontend && bun run dev

bots:
  cd apps/bots && cargo run

compose:
  docker compose up --build

# ================================

db-run:
  docker compose up -d postgres clickhouse

db-reset:
  # assumes exchange database is already created from docker compose
  psql $DATABASE_URL -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" 2>/dev/null || true
  clickhouse client --user default --password password --query "DROP TABLE IF EXISTS exchange.candles" 2>/dev/null || true
  just db-setup

db-setup:
  cd apps/backend/src/db/pg && cargo sqlx migrate run --database-url $DATABASE_URL
  clickhouse client --user default --password password --query "$(cat src/db/ch/schema.sql)"

db-prepare:
  cd apps/backend && cargo sqlx prepare --database-url $DATABASE_URL

# ================================

install:
  cd apps/frontend && bun install
  cargo build --workspace
  cd packages/sdk-python && uv sync

test:
  cargo test --workspace

bench:
  cd apps/backend && cargo bench
  open target/criterion/report/index.html

# ================================

openapi:
  cd apps/backend && cargo run --bin generate_openapi
  cd packages/sdk-typescript && bun run generate

fmt:
  cd apps/frontend && bun run format
  cargo fmt --all

lint:
  cd apps/frontend && bun run lint
  cargo clippy --workspace --all-targets

typecheck:
  cd apps/frontend && bun run typecheck

clean:
  cd apps/frontend && bun run clean
  cargo clean

ci:
  just openapi
  just fmt
  just lint
  just db-prepare



