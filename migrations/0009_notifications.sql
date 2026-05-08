-- Notifications

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    recipient_did TEXT NOT NULL,
    reason TEXT NOT NULL,
    subject_uri TEXT NOT NULL,
    read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient ON notifications(recipient_did);
CREATE INDEX IF NOT EXISTS idx_notifications_unread ON notifications(recipient_did, read);
