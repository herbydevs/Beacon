#!/bin/bash

# 1. Define the connection URI
PG_URI="postgres://user:password@localhost:5436/beacon"

echo "Connecting to PostgreSQL and initializing 'servers' table..."

# 2. Run the SQL commands using psql
# We use a 'here-document' (<<EOF) to keep the SQL readable
psql "$PG_URI" <<EOF
-- Get rid of the old table
DROP TABLE IF EXISTS servers;

-- Create the UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create the new table
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL UNIQUE,
    container_id TEXT NOT NULL,
    version TEXT NOT NULL,
    status TEXT NOT NULL,
    server_type TEXT NOT NULL
);
EOF

# 3. Check if the command succeeded
if [ $? -eq 0 ]; then
    echo "✅ Database setup successful."
else
    echo "❌ An error occurred during database setup."
    exit 1
fi