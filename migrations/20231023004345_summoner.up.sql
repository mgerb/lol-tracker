-- Add up migration script here
CREATE TABLE IF NOT EXISTS summoner (
    id TEXT NOT NULL PRIMARY KEY,
    guild_id INTEGER NOT NULL,
    name TEXT COLLATE NOCASE NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    solo_tier TEXT,
    solo_lp INTEGER,
    solo_division INTEGER,
    flex_tier TEXT,
    flex_lp INTEGER,
    flex_division INTEGER,

    FOREIGN KEY (guild_id) REFERENCES guild (id)
);

CREATE TRIGGER [SetUpdatedAt_summoner]
    AFTER UPDATE
    ON summoner 
    FOR EACH ROW
BEGIN
    UPDATE summoner SET updated_at = (strftime('%s', 'now')) WHERE updated_at = old.updated_at;
END
