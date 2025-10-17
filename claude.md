## Project Overview

This is a **cryptocurrency exchange** built with:

- **Backend**: Rust (Axum) with WebSocket support
- **Frontend**: Next.js with TypeScript
- **Database**: PostgreSQL
- **Architecture**: Monorepo with shared schemas

## Project Structure

```
exchange/
├── apps/
│   ├── backend/          # Rust Axum API server
│   │   ├── src/
│   │   │   ├── api/
│   │   │   │   ├── rest/     # REST endpoints
│   │   │   │   └── ws/       # WebSocket handlers
│   │   │   ├── models.rs     # Data structures
│   │   │   ├── lib.rs        # Library entry point
│   │   │   └── main.rs       # Binary entry point
│   │   └── scripts/
│   │       └── generate_openapi.rs
│   └── frontend/         # Next.js React app
│       ├── src/
│       │   ├── app/          # Next.js app router
│       │   ├── components/   # React components
│       │   └── lib/          # Utilities and types
│       └── public/vendor/trading-view/  # TradingView integration
├── packages/
│   └── shared/           # Shared schemas and types
│       ├── openapi.json  # REST API schema
│       └── websocket-schema.json  # WebSocket message schema
└── justfile             # Build commands
```

## Key Concepts

### Module System (Rust)

- **`lib.rs`**: Library entry point, defines public API
- **`mod.rs`**: Module entry point for directories
- **`main.rs`**: Binary entry point for the server
- **Module paths**: `backend::api::rest::health::health_check`

### API Architecture

- **REST API**: Standard HTTP endpoints with OpenAPI documentation
- **WebSocket**: Real-time communication for trading data
- **Shared Schemas**: Type-safe communication between frontend/backend

### Development Workflow

- **Backend**: `just run-backend` or `cargo run`
- **Frontend**: `just run-frontend` or `bun run dev`
- **Schema Generation**: `cargo run --bin generate-openapi`

## Common Tasks

### Adding New REST Endpoints

1. **Create handler function** in `apps/backend/src/api/rest/`
2. **Add route** in `apps/backend/src/api/rest/mod.rs`
3. **Update OpenAPI** in `apps/backend/src/lib.rs`
4. **Regenerate schema**: `cargo run --bin generate-openapi`

### Adding WebSocket Messages

1. **Define message types** in `apps/backend/src/models.rs`
2. **Create handler** in `apps/backend/src/api/ws/`
3. **Update schema** in `packages/shared/websocket-schema.json`
4. **Generate TypeScript types** for frontend

### Database Operations

- **Migrations**: `apps/backend/src/db/migrations/`
- **Models**: Define in `apps/backend/src/models.rs`
- **Queries**: Use SQLx or similar ORM

## Code Patterns

### Rust Backend Patterns

```rust
// Module declaration in lib.rs
pub mod api;
pub mod models;

// Route creation
pub fn create_routes() -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
}

// WebSocket handler
async fn websocket_handler(ws: WebSocket) {
    // Handle WebSocket connection
}
```

### Frontend Patterns

```typescript
// Type-safe API calls
import { ApiResponse } from "@/lib/types/openapi";

// WebSocket client
const ws = new WebSocket("ws://localhost:8001/ws");
```

## Useful Commands

```bash
# Development
just run-backend          # Start backend server
just run-frontend         # Start frontend dev server
just install              # Install all dependencies

# Building
just fmt                  # Format all code
just lint                 # Lint all code
just test                 # Run all tests
just clean                # Clean build artifacts

# Schema generation
cargo run --bin generate-openapi  # Generate OpenAPI spec
```
