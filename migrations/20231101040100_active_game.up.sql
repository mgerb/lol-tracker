-- Add up migration script here
CREATE TABLE IF NOT EXISTS active_game (
    id TEXT NOT NULL PRIMARY KEY,
    summoner_id TEXT COLLATE NOCASE NOT NULL,
    game_created_at INTEGER NOT NULL, -- riot timestamp
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    champion TEXT NOT NULL,
    role TEXT NOT NULL,
    spectate_link TEXT NOT NULL,
    notified boolean NOT NULL,
    game_mode TEXT NOT NULL
);
