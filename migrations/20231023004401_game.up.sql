-- Add up migration script here
CREATE TABLE IF NOT EXISTS game (
    id TEXT NOT NULL PRIMARY KEY,
    summoner_id TEXT COLLATE NOCASE NOT NULL,
    game_created_at INTEGER NOT NULL, -- riot timestamp
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    assists INTEGER NOT NULL,
    deaths INTEGER NOT NULL,
    kills INTEGER NOT NULL,
    win BOOLEAN NOT NULL,
    notified BOOLEAN NOT NULL DEFAULT 0,
    champion_name TEXT NOT NULL,
    game_mode TEXT NOT NULL,
    lp_change INTEGER,
    promotion_text TEXT,

    FOREIGN KEY (summoner_id) REFERENCES summoner (id)
);

CREATE TRIGGER [SetUpdatedAt_game]
    AFTER UPDATE
    ON game 
    FOR EACH ROW
BEGIN
    UPDATE game SET updated_at = (strftime('%s', 'now')) WHERE updated_at = old.updated_at;
END
