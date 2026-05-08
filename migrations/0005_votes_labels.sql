-- Votes and labels

CREATE TABLE IF NOT EXISTS votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vote_uri TEXT NOT NULL,
    subject_uri TEXT NOT NULL,
    voter_did TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(subject_uri, voter_did)
);
CREATE INDEX IF NOT EXISTS idx_votes_subject ON votes(subject_uri);

CREATE TABLE IF NOT EXISTS labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label_uri TEXT NOT NULL,
    subject_uri TEXT NOT NULL,
    val TEXT NOT NULL,
    src_did TEXT NOT NULL,
    neg INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_labels_subject ON labels(subject_uri);
