@echo off
SET CONTAINER_NAME=beacon_db_prod

echo 📡 Target container: %CONTAINER_NAME%
echo Initializing 'servers' table...

:: 1. Check if the container is actually running
docker ps --filter "name=%CONTAINER_NAME%" --format "{{.Names}}" | findstr /I "%CONTAINER_NAME%" >nul
if %errorlevel% neq 0 (
    echo ❌ ERROR: Container %CONTAINER_NAME% is not running.
    echo Please start your Docker stack first.
    pause
    exit /b 1
)

:: 2. Execute SQL inside the container
:: We pass the SQL as a single string to the -c flag for maximum compatibility with Batch
docker exec -i %CONTAINER_NAME% psql -U user -d beacon -c "DROP TABLE IF EXISTS servers; CREATE TABLE servers (id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name TEXT NOT NULL UNIQUE, container_id TEXT NOT NULL, version TEXT NOT NULL, status TEXT NOT NULL, server_type TEXT NOT NULL, created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP); \dt servers"

:: 3. Final Check
if %errorlevel% equ 0 (
    echo --------------------------------------------------
    echo ✅ SUCCESS: Database table 'servers' is ready.
    echo --------------------------------------------------
) else (
    echo --------------------------------------------------
    echo ❌ ERROR: SQL execution failed inside the container.
    echo Check if the database 'beacon' and user 'user' exist.
    echo --------------------------------------------------
    pause
    exit /b 1
)

pause