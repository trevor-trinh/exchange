# Stage 1: Builder
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy workspace root package files first
COPY package.json bun.lockb* ./

# Copy workspace member package files
COPY apps/frontend/package.json ./apps/frontend/
COPY packages/sdk-typescript/package.json ./packages/sdk-typescript/

# Install dependencies (workspace aware)
RUN bun install --frozen-lockfile

# Copy shared schemas (needed for SDK generation)
COPY packages/shared ./packages/shared

# Copy SDK source
COPY packages/sdk-typescript ./packages/sdk-typescript

# Build the SDK first (generates types and compiles)
RUN cd packages/sdk-typescript && bun run build

# Copy frontend source
COPY apps/frontend ./apps/frontend

# Set working directory to frontend app
WORKDIR /app/apps/frontend

ARG NEXT_PUBLIC_API_URL
ARG NEXT_PUBLIC_WS_URL

# Set environment variables for Next.js build
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_WS_URL=$NEXT_PUBLIC_WS_URL

ENV NEXT_TELEMETRY_DISABLED=1
RUN bun run build

# Stage 2: Runtime
FROM oven/bun:1-slim

WORKDIR /app

# Copy built application from builder (monorepo structure)
COPY --from=builder /app/apps/frontend/.next/standalone ./
COPY --from=builder /app/apps/frontend/.next/static ./apps/frontend/.next/static
COPY --from=builder /app/apps/frontend/public ./apps/frontend/public

ENV HOSTNAME=0.0.0.0
ENV PORT=3000

EXPOSE 3000

# In monorepo, server.js is at apps/frontend/server.js within standalone output
CMD ["bun", "apps/frontend/server.js"]
