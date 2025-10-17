# syntax=docker/dockerfile:1

# Stage 1: Builder
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lockb* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Build
ENV NEXT_TELEMETRY_DISABLED=1
RUN bun run build

# Stage 2: Runtime
FROM oven/bun:1-slim

WORKDIR /app

# Copy built application
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/node_modules ./node_modules

# Copy env defaults
COPY .env.defaults .env.defaults

EXPOSE 3000

CMD ["bun", "run", "start"]
