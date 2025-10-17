default:
  just --list

run-frontend:
  cd apps/frontend && bun run dev

run-backend:
  cd apps/backend && cargo run

# ================================

test:
  cd apps/backend && cargo test

# ================================

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



