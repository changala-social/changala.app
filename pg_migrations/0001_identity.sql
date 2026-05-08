CREATE TABLE IF NOT EXISTS otp_codes (
    id BIGSERIAL PRIMARY KEY,
    did TEXT NOT NULL,
    email TEXT NOT NULL,
    code TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);
CREATE INDEX IF NOT EXISTS idx_otp_did_email ON otp_codes(did, email);

CREATE TABLE IF NOT EXISTS memberships (
    id BIGSERIAL PRIMARY KEY,
    did TEXT NOT NULL,
    institution_did TEXT NOT NULL,
    institution_domain TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'student',
    membership_uri TEXT,
    verified_email TEXT,
    verified_at TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    UNIQUE(did, institution_did)
);
CREATE INDEX IF NOT EXISTS idx_memberships_did ON memberships(did);
