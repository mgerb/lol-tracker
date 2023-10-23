-- Add up migration script here
CREATE TABLE IF NOT EXISTS summoner (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
