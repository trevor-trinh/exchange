# Docker Setup

## Quick Start

```bash
# Start all services
docker compose up -d

# View logs
docker compose logs -f

# Stop all services
docker compose down

# Rebuild after code changes
docker compose up -d --build
```

## Services

| Service | Port | URL |
|---------|------|-----|
| Backend | 8888 | http://localhost:8888 |
| Frontend | 3000 | http://localhost:3000 |
| Postgres | 5432 | postgresql://postgres:password@localhost:5432/exchange |
| ClickHouse | 8123 | http://localhost:8123 |

## Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Frontend    │────→│   Backend    │────→│  Postgres    │
│  (Bun)       │     │   (Rust)     │     │ (TimescaleDB)│
│  Port 3000   │     │   Port 8888  │     │  Port 5432   │
└──────────────┘     └──────┬───────┘     └──────────────┘
                            │
                     ┌──────┴───────┐
                     │              │
              ┌──────▼──────┐  ┌───▼─────────┐
              │ ClickHouse  │  │ Market Bot  │
              │ Port 8123   │  │ (Rust)      │
              └─────────────┘  └─────────────┘
```

## Dockerfiles

### Backend (Rust)
- Two-stage build
- Stage 1: Build app with Rust toolchain
- Stage 2: Runtime (slim Debian image)

### Frontend (Bun)
- Two-stage build for smaller image
- Stage 1: Build Next.js app
- Stage 2: Runtime (bun:1-slim with built assets)

### Bots (Rust)
- Same as backend

## Environment Variables

### Development (Local)
Each service loads `.env.defaults` → `.env` (optional override)

```bash
# Backend
cd apps/backend && cargo run

# Frontend
cd apps/frontend && bun run dev

# Bot
cd apps/bots && cargo run
```

### Docker
Environment variables hardcoded in `docker-compose.yaml`

## Dev Workflow (Local)

For hot reload during development, run services locally:

```bash
# 1. Start databases only
docker compose up postgres clickhouse -d

# 2. Run backend (hot reload with cargo watch)
cd apps/backend && cargo watch -x run

# 3. Run frontend (hot reload)
cd apps/frontend && bun dev

# 4. Optional: Run bot
cd apps/bots && cargo run
```

## Dev Workflow (Full Docker)

```bash
# Start everything
docker compose up -d

# Rebuild after code changes
docker compose up -d --build

# View logs
docker compose logs -f backend frontend
```
