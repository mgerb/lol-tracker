-- Add up migration script here
CREATE TABLE IF NOT EXISTS log (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    message TEXT NOT NULL,
    error_type TEXT NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
)
