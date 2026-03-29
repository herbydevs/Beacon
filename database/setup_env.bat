@echo off
set "TARGET_DIR=..\"
set "ENV_FILE=%TARGET_DIR%\.env"

echo Creating .env file at %ENV_FILE%...

(
echo POSTGRES_USER=user
echo POSTGRES_PASSWORD=password
echo POSTGRES_DB=beacon
echo DATABASE_URL=postgres://user:password@postgres:5432/beacon
echo REDIS_HOST=localhost
echo KEYCLOAK_SERVER_URL=http://localhost:8080
echo KEYCLOAK_REALM=beacon
echo KEYCLOAK_CLIENT_ID=beacon-backend
echo KEYCLOAK_CLIENT_SECRET=your_secret_here
echo KEYCLOAK_REDIRECT_URI=http://localhost:8000/api/v1/auth/callback
echo KC_BOOTSTRAP_ADMIN_USERNAME=admin
echo KC_BOOTSTRAP_ADMIN_PASSWORD=pass
echo BACKEND_PORT=8000
echo RUST_LOG=beacon_backend=debug,axum=info,tower_http=debug
echo DOCKER_HOST=unix:///var/run/docker.sock
) > "%ENV_FILE%"

echo Done!
pause