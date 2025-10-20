default:
  just --list

backend:
  cd apps/backend && cargo run --release

frontend:
  cd apps/frontend && bun run dev

compose:
  docker compose up --build

# ================================

db-run:
  docker compose up -d postgres clickhouse

db-reset:
  just postgres-reset
  just clickhouse-reset

postgres-reset:
  # assumes database exists from docker compose
  psql postgresql://postgres:password@localhost:5432/exchange -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
  just postgres-migrate

clickhouse-reset:
  # assumes database exists from docker compose
  clickhouse client --user default --password password --query "DROP TABLE IF EXISTS exchange.candles"
  clickhouse client --user default --password password --database exchange --queries-file apps/backend/src/db/clickhouse/schema.sql

postgres-migrate:
  cd apps/backend/src/db/postgres && sqlx migrate run --database-url postgresql://postgres:password@localhost:5432/exchange

# ================================

build:
  cd apps/frontend && bun install
  cd apps/backend && cargo build

test:
  cd apps/backend && cargo test


# ================================

openapi:
  cd apps/backend && cargo run --bin generate-openapi
  cd apps/frontend && bun run generate-openapi

fmt:
  cd apps/frontend && bun run format
  cd apps/backend && cargo fmt

lint:
  cd apps/frontend && bun run lint
  cd apps/backend && cargo clippy

typecheck:
  cd apps/frontend && bun run typecheck

clean:
  cd apps/frontend && bun run clean
  cd apps/backend && cargo clean

ci:
  just openapi
  just fmt
  just lint



