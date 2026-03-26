# --- STAGE 1: Build Frontend ---
FROM node:20-alpine AS frontend-builder
WORKDIR /app/beacon-frontend
COPY beacon-frontend/package*.json ./
RUN npm install
COPY beacon-frontend/ .
RUN npm run build

# --- STAGE 2: Build Backend ---
FROM rust:1.75-slim AS backend-builder
WORKDIR /app/backend
# Install system dependencies for SQLx and OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY backend/Cargo.toml backend/Cargo.lock ./
# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
COPY backend/src ./src
RUN touch src/main.rs && cargo build --release

# --- STAGE 3: Final Runtime ---
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy backend binary
COPY --from=backend-builder /app/backend/target/release/beacon-backend ./beacon

# Copy frontend static files for Axum to serve
COPY --from=frontend-builder /app/beacon-frontend/dist ./static

# Expose the Axum port
EXPOSE 8000

CMD ["./beacon"]