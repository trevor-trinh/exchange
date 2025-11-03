export DATABASE_URL := "postgresql://postgres:password@localhost:5432/exchange"

default:
  just --list

backend:
  cd apps/backend && cargo run

frontend:
  just types
  bun run dev

bots:
  cd apps/bots && cargo run

compose:
  docker compose up --build

# ================================

db-run:
  docker compose up -d postgres clickhouse

db:
  just db-reset
  just db-prepare
  just db-setup
  just db-init

db-init:
  # inits exchange with tokens and markets from config.toml
  # this also automatically sets up database schemas
  cd apps/backend && cargo run --bin init_exchange

db-reset:
  # Drop and recreate databases (WARNING: destroys all data)
  psql $DATABASE_URL -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" 2>/dev/null || true
  clickhouse client --user default --password password --query "DROP DATABASE IF EXISTS exchange" 2>/dev/null || true
  just db-setup

db-setup:
  cd apps/backend/src/db/pg && cargo sqlx migrate run --database-url $DATABASE_URL
  clickhouse client --user default --password password --query "$(cat apps/backend/src/db/ch/schema.sql)"

db-prepare:
  cd apps/backend && cargo sqlx prepare --database-url $DATABASE_URL

# ================================

install:
  bun install
  cargo build --workspace
  cd packages/sdk-python && uv sync

test:
  cargo test --workspace

bench:
  cd apps/backend && cargo bench
  open target/criterion/report/index.html

# ================================

types:
  cd apps/backend && cargo run --bin generate_openapi
  cargo run -p schema-generator
  bun --filter @exchange/sdk generate
  just types-python

types-python:
  #!/usr/bin/env bash
  cd packages/sdk-python && \
  mkdir -p exchange_sdk/generated && \
  uv run --with datamodel-code-generator[http] datamodel-codegen \
    --input ../../packages/shared/websocket.json \
    --input-file-type jsonschema \
    --output exchange_sdk/generated/websocket.py \
    --output-model-type pydantic_v2.BaseModel \
    --use-union-operator \
    --field-constraints \
    --use-standard-collections \
    --target-python-version 3.10 && \
  echo "# Generated WebSocket types" > exchange_sdk/generated/__init__.py && \
  echo "from .websocket import *" >> exchange_sdk/generated/__init__.py

fmt:
  bun run format
  cargo fmt --all
  just sort

lint:
  bun run lint
  cargo clippy --workspace --all-targets

typecheck:
  bun run typecheck

clean:
  bun run clean
  cargo clean

ci:
  just install
  just types
  just fmt
  just lint
  just db-prepare

sort:
  cargo sort -g -w

