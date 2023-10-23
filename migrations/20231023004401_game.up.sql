-- Add up migration script here
CREATE TABLE IF NOT EXISTS game (
    id TEXT NOT NULL PRIMARY KEY,
    summoner_id TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    champion_id INTEGER NOT NULL,
    assists INTEGER NOT NULL,
    deaths INTEGER NOT NULL,
    kills INTEGER NOT NULL,
    result TEXT NOT NULL,
    division INTEGER,
    lp INTEGER,
    tier TEXT,
    tier_image_url TEXT,
    border_image_url TEXT,

    FOREIGN KEY (summoner_id) REFERENCES summoner (id)
)
