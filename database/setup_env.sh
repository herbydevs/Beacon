#!/bin/bash

TARGET_DIR="../../"
ENV_FILE="${TARGET_DIR}.env"

echo "Creating .env file at $ENV_FILE..."

cat <<EOF > "$ENV_FILE"
POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_DB=beacon
DATABASE_URL=postgres://user:password@postgres:5432/beacon
REDIS_HOST=localhost
KEYCLOAK_SERVER_URL=http://localhost:8080
KEYCLOAK_REALM=beacon
KEYCLOAK_CLIENT_ID=beacon-backend
KEYCLOAK_CLIENT_SECRET=your_secret_here
KEYCLOAK_REDIRECT_URI=http://localhost:8000/api/v1/auth/callback
KC_BOOTSTRAP_ADMIN_USERNAME=admin
KC_BOOTSTRAP_ADMIN_PASSWORD=pass
BACKEND_PORT=8000
RUST_LOG=beacon_backend=debug,axum=info,tower_http=debug
DOCKER_HOST=unix:///var/run/docker.sock
EOF

echo "Done!"
chmod +x "$ENV_FILE" 2>/dev/null