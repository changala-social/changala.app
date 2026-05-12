-- Migrate api_keys table to atrg-auth v0.2.2 standard schema.
-- Changes: TEXT timestamps → BIGINT, JSON scopes → comma-separated, new hash format.
-- Strategy: drop and recreate (MVP — no production data to preserve yet).

DROP TABLE IF EXISTS api_keys;

CREATE TABLE IF NOT EXISTS api_keys (
    id           BIGSERIAL PRIMARY KEY,
    key_hash     TEXT NOT NULL UNIQUE,
    key_prefix   TEXT NOT NULL,
    did          TEXT NOT NULL,
    name         TEXT NOT NULL,
    scopes       TEXT NOT NULL DEFAULT '',
    expires_at   BIGINT,
    created_at   BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
    last_used_at BIGINT
);
CREATE INDEX IF NOT EXISTS idx_api_keys_did ON api_keys(did);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_prefix ON api_keys(key_prefix);
