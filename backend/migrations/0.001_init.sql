-- Create the users table
CREATE TABLE IF NOT EXISTS users (
                                     id TEXT PRIMARY KEY,        -- Keycloak UUID
                                     username TEXT NOT NULL,
                                     join_date BIGINT NOT NULL,
                                     servers TEXT[] NOT NULL DEFAULT '{}'
);