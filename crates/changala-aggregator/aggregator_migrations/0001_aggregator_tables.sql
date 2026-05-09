-- Aggregator materialised tables
-- These mirror Ring tables but are populated via firehose events, not direct writes.

CREATE TABLE IF NOT EXISTS courses (
    id BIGSERIAL PRIMARY KEY,
    uri TEXT UNIQUE,
    rkey TEXT,
    title TEXT NOT NULL,
    code TEXT NOT NULL,
    department TEXT,
    semester TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'institution',
    created_by TEXT NOT NULL,
    class_rep_did TEXT,
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS enrollments (
    id BIGSERIAL PRIMARY KEY,
    course_uri TEXT NOT NULL,
    did TEXT NOT NULL,
    enrolled_at TEXT NOT NULL,
    UNIQUE(course_uri, did)
);

CREATE TABLE IF NOT EXISTS sessions (
    id BIGSERIAL PRIMARY KEY,
    uri TEXT UNIQUE,
    rkey TEXT,
    course_uri TEXT NOT NULL,
    scheduled_at TEXT NOT NULL,
    duration_mins INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    created_by TEXT NOT NULL,
    topic TEXT,
    opened_at TEXT,
    closed_at TEXT,
    keyword_window_expires_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS keywords (
    id BIGSERIAL PRIMARY KEY,
    session_uri TEXT NOT NULL,
    did TEXT NOT NULL,
    text TEXT NOT NULL,
    keyword_uri TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    id BIGSERIAL PRIMARY KEY,
    uri TEXT,
    session_uri TEXT NOT NULL,
    author_did TEXT NOT NULL,
    format TEXT NOT NULL,
    ring_did TEXT NOT NULL,
    cid TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    parent_note_uri TEXT,
    summary TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS votes (
    id BIGSERIAL PRIMARY KEY,
    vote_uri TEXT NOT NULL,
    subject_uri TEXT NOT NULL,
    voter_did TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(subject_uri, voter_did)
);

CREATE TABLE IF NOT EXISTS labels (
    id BIGSERIAL PRIMARY KEY,
    label_uri TEXT NOT NULL,
    subject_uri TEXT NOT NULL,
    val TEXT NOT NULL,
    src_did TEXT NOT NULL,
    neg BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS brain_nodes (
    id BIGSERIAL PRIMARY KEY,
    uri TEXT UNIQUE,
    author_did TEXT NOT NULL,
    title TEXT NOT NULL,
    format TEXT NOT NULL DEFAULT 'markdown',
    ring_did TEXT NOT NULL,
    cid TEXT NOT NULL,
    tags TEXT NOT NULL DEFAULT '[]',
    academic_ref TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    summary TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS brain_links (
    id BIGSERIAL PRIMARY KEY,
    link_uri TEXT UNIQUE,
    from_uri TEXT NOT NULL,
    to_uri TEXT NOT NULL,
    label TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    recipient_did TEXT NOT NULL,
    reason TEXT NOT NULL,
    subject_uri TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS archives (
    id BIGSERIAL PRIMARY KEY,
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

CREATE TABLE IF NOT EXISTS memberships (
    id BIGSERIAL PRIMARY KEY,
    did TEXT NOT NULL,
    institution_did TEXT NOT NULL,
    institution_domain TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'student',
    membership_uri TEXT,
    verified_email TEXT,
    verified_at TEXT NOT NULL,
    UNIQUE(did, institution_did)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_agg_notes_session ON notes(session_uri);
CREATE INDEX IF NOT EXISTS idx_agg_keywords_session ON keywords(session_uri);
CREATE INDEX IF NOT EXISTS idx_agg_votes_subject ON votes(subject_uri);
CREATE INDEX IF NOT EXISTS idx_agg_labels_subject ON labels(subject_uri);
CREATE INDEX IF NOT EXISTS idx_agg_brain_nodes_author ON brain_nodes(author_did);
CREATE INDEX IF NOT EXISTS idx_agg_brain_links_from ON brain_links(from_uri);
CREATE INDEX IF NOT EXISTS idx_agg_brain_links_to ON brain_links(to_uri);
CREATE INDEX IF NOT EXISTS idx_agg_notifications_recipient ON notifications(recipient_did);
CREATE INDEX IF NOT EXISTS idx_agg_enrollments_course ON enrollments(course_uri);
CREATE INDEX IF NOT EXISTS idx_agg_memberships_did ON memberships(did);
