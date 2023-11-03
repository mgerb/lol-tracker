-- Add up migration script here
CREATE TABLE IF NOT EXISTS summoner (
    id TEXT COLLATE NOCASE NOT NULL PRIMARY KEY,
    guild_id INTEGER NOT NULL,
    name TEXT COLLATE NOCASE NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now')),
    queue_type TEXT,
    tier TEXT,
    lp INTEGER,
    division TEXT,
    icon_url TEXT NOT NULL,

    FOREIGN KEY (guild_id) REFERENCES guild (id)
);

CREATE TRIGGER [SetUpdatedAt_summoner]
    AFTER UPDATE
    ON summoner 
    FOR EACH ROW
BEGIN
    UPDATE summoner SET updated_at = (strftime('%s', 'now')) WHERE updated_at = old.updated_at;
END
