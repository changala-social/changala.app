-- Archives

CREATE TABLE IF NOT EXISTS archives (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    archive_uri TEXT,
    course_uri TEXT NOT NULL,
    semester TEXT NOT NULL,
    sealed_by TEXT,
    ring_did TEXT,
    cid TEXT,
    internet_archive_url TEXT,
    session_count INTEGER NOT NULL DEFAULT 0,
    note_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'initiated',
    initiated_at TEXT NOT NULL,
    sealed_at TEXT,
    UNIQUE(course_uri, semester)
);
