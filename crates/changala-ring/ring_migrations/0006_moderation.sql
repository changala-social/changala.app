CREATE TABLE IF NOT EXISTS bans (
    id BIGSERIAL PRIMARY KEY,
    target_did TEXT NOT NULL UNIQUE,
    permanent BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TEXT,
    reason TEXT,
    banned_at TEXT NOT NULL
);
