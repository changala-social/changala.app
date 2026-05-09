CREATE TABLE IF NOT EXISTS sessions (
    id BIGSERIAL PRIMARY KEY,
    uri TEXT NOT NULL UNIQUE,
    rkey TEXT NOT NULL UNIQUE,
    course_uri TEXT NOT NULL REFERENCES courses(uri),
    scheduled_at TEXT NOT NULL,
    duration_mins INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    created_by TEXT NOT NULL,
    topic TEXT,
    opened_at TEXT,
    closed_at TEXT,
    keyword_window_expires_at TEXT,
    rescheduled_to TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sessions_course ON sessions(course_uri);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

CREATE TABLE IF NOT EXISTS keywords (
    id BIGSERIAL PRIMARY KEY,
    session_uri TEXT NOT NULL REFERENCES sessions(uri),
    did TEXT NOT NULL,
    text TEXT NOT NULL,
    keyword_uri TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(session_uri, did, text)
);
CREATE INDEX IF NOT EXISTS idx_keywords_session ON keywords(session_uri);
