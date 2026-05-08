-- Notes, collective notes, and edit proposals

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uri TEXT,
    session_uri TEXT NOT NULL REFERENCES sessions(uri),
    author_did TEXT NOT NULL,
    format TEXT NOT NULL,
    ring_did TEXT NOT NULL,
    cid TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    parent_note_uri TEXT,
    summary TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_notes_session ON notes(session_uri);
CREATE INDEX IF NOT EXISTS idx_notes_author ON notes(author_did);

CREATE TABLE IF NOT EXISTS collective_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_uri TEXT NOT NULL UNIQUE REFERENCES sessions(uri),
    ring_did TEXT NOT NULL,
    cid TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS edit_proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_uri TEXT,
    session_uri TEXT NOT NULL REFERENCES sessions(uri),
    proposer_did TEXT NOT NULL,
    diff_ring_did TEXT NOT NULL,
    diff_cid TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    summary TEXT,
    resolved_at TEXT,
    resolved_by TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_proposals_session ON edit_proposals(session_uri);
CREATE INDEX IF NOT EXISTS idx_proposals_status ON edit_proposals(status);
