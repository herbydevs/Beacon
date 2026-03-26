-- Create the users table
CREATE TABLE IF NOT EXISTS users (
                                     id TEXT PRIMARY KEY,        -- Keycloak UUID
                                     username TEXT NOT NULL,
                                     join_date BIGINT NOT NULL,
                                     servers TEXT[] NOT NULL DEFAULT '{}'
);



CREATE TABLE IF NOT EXISTS servers (
                                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    container_id TEXT UNIQUE,
    version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'creating',
    created_at TIMESTAMPTZ DEFAULT now()
    );