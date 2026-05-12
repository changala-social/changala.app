-- atrg-email OTP verification table (replaces custom otp_codes table)
CREATE TABLE IF NOT EXISTS atrg_otp_codes (
    id BIGSERIAL PRIMARY KEY,
    did TEXT NOT NULL,
    email TEXT NOT NULL,
    code TEXT NOT NULL,
    expires_at BIGINT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::bigint
);
CREATE INDEX IF NOT EXISTS idx_atrg_otp_did_email ON atrg_otp_codes(did, email);
