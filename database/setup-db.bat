@echo off
SETLOCAL

:: Set the connection URI
SET "PG_URI=postgres://user:password@localhost:5436/beacon"

echo Connecting to database and initializing 'servers' table...

:: We pass the SQL commands directly via the -c flag
:: Note: We wrap the SQL in quotes. 
psql "%PG_URI%" -c "DROP TABLE IF EXISTS servers; CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"; CREATE TABLE servers (id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), name TEXT NOT NULL UNIQUE, container_id TEXT NOT NULL, version TEXT NOT NULL, status TEXT NOT NULL, server_type TEXT NOT NULL);"

if %ERRORLEVEL% EQU 0 (
    echo Database setup successfully.
) else (
    echo An error occurred during database setup.
)

PAUSE
ENDLOCAL