CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    actor_did TEXT NOT NULL,
    action TEXT NOT NULL,
    target_did TEXT,
    details JSONB,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor ON audit_log(actor_did);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at);
