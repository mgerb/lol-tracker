-- Add up migration script here
CREATE TABLE IF NOT EXISTS guild (
    id INTEGER NOT NULL PRIMARY KEY,
    chat_channel_id INTEGER,
    name TEXT COLLATE NOCASE NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER DEFAULT (strftime('%s', 'now'))
);

CREATE TRIGGER [SetUpdatedAt_guild]
    AFTER UPDATE
    ON guild
    FOR EACH ROW
BEGIN
    UPDATE guild SET updated_at = (strftime('%s', 'now')) WHERE updated_at = old.updated_at;
END
