#!/bin/bash

# 1. Dynamically find the Postgres container name
# This looks for a running container with 'postgres' in the name
CONTAINER_NAME="beacon_db_prod"

if [ -z "$CONTAINER_NAME" ]; then
    echo "❌ Error: No Postgres container found running. Start your stack first!"
    exit 1
fi

echo "📡 Found container: $CONTAINER_NAME"
echo "Initializing 'servers' table..."

# 2. Execute psql INSIDE the container
# We use -i (interactive) to pipe the SQL commands into the container's stdin
docker exec -i "$CONTAINER_NAME" psql -U user -d beacon <<EOF
-- Stop execution if any command fails
\set ON_ERROR_STOP on

-- Cleanup old data
DROP TABLE IF EXISTS servers;

-- Create table with native UUID generation (Postgres 13+)
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    container_id TEXT NOT NULL,
    version TEXT NOT NULL,
    status TEXT NOT NULL,
    server_type TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Show confirmation
\dt servers
EOF

# 3. Final Check
if [ $? -eq 0 ]; then
    echo "--------------------------------------------------"
    echo "✅ SUCCESS: Database table 'servers' is ready."
    echo "--------------------------------------------------"
else
    echo "--------------------------------------------------"
    echo "❌ ERROR: SQL execution failed inside the container."
    echo "Check if the database 'beacon' and user 'user' exist."
    echo "--------------------------------------------------"
    exit 1
fi