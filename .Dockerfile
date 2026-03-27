
FROM rustlang/rust:nightly-slim AS backend-builder


ENV RUSTC_BOOTSTRAP=1

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./

# Standard build caching...
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src

# Ensure all subdirectories (like dbmodels) are copied
COPY backend/src ./src

# Final build - it will now ignore the missing database
RUN touch src/main.rs && cargo build --release

# --- STAGE 3: Final Runtime ---
FROM debian:bookworm-slim
WORKDIR /app

# Added libpq5 for Postgres runtime support
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# FIX: Path must match your [package] name "backend"
COPY --from=backend-builder /app/backend/target/release/backend ./beacon
COPY --from=frontend-builder /app/beacon-frontend/dist ./static

RUN chmod +x ./beacon

EXPOSE 8000
CMD ["./beacon"]