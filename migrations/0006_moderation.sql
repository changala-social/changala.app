-- Moderation deny list

CREATE TABLE IF NOT EXISTS bans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_did TEXT NOT NULL UNIQUE,
    permanent INTEGER NOT NULL DEFAULT 1,
    expires_at TEXT,
    reason TEXT,
    banned_at TEXT NOT NULL
);
