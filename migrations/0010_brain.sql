-- Brain nodes and links

CREATE TABLE IF NOT EXISTS brain_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uri TEXT,
    author_did TEXT NOT NULL,
    title TEXT NOT NULL,
    format TEXT NOT NULL DEFAULT 'markdown',
    ring_did TEXT NOT NULL,
    cid TEXT NOT NULL,
    tags TEXT,
    academic_ref TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    parent_node_uri TEXT,
    summary TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_brain_nodes_author ON brain_nodes(author_did);
CREATE INDEX IF NOT EXISTS idx_brain_nodes_academic_ref ON brain_nodes(academic_ref);

CREATE TABLE IF NOT EXISTS brain_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    link_uri TEXT,
    from_uri TEXT NOT NULL,
    to_uri TEXT NOT NULL,
    label TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(from_uri, to_uri)
);
CREATE INDEX IF NOT EXISTS idx_brain_links_from ON brain_links(from_uri);
CREATE INDEX IF NOT EXISTS idx_brain_links_to ON brain_links(to_uri);
