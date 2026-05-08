-- Identity: OTP codes and verified memberships

CREATE TABLE IF NOT EXISTS otp_codes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    did TEXT NOT NULL,
    email TEXT NOT NULL,
    code TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    used INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX IF NOT EXISTS idx_otp_did_email ON otp_codes(did, email);

CREATE TABLE IF NOT EXISTS memberships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    did TEXT NOT NULL,
    institution_did TEXT NOT NULL,
    institution_domain TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'student',
    membership_uri TEXT,
    verified_email TEXT,
    verified_at TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(did, institution_did)
);
CREATE INDEX IF NOT EXISTS idx_memberships_did ON memberships(did);
