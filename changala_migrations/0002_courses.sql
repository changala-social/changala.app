CREATE TABLE IF NOT EXISTS courses (
    id BIGSERIAL PRIMARY KEY,
    uri TEXT NOT NULL UNIQUE,
    rkey TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    code TEXT NOT NULL,
    department TEXT NOT NULL,
    semester TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'institution',
    created_by TEXT NOT NULL,
    class_rep_did TEXT,
    description TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(code, semester)
);
CREATE INDEX IF NOT EXISTS idx_courses_semester ON courses(semester);
CREATE INDEX IF NOT EXISTS idx_courses_department ON courses(department);

CREATE TABLE IF NOT EXISTS enrollments (
    id BIGSERIAL PRIMARY KEY,
    course_uri TEXT NOT NULL REFERENCES courses(uri),
    did TEXT NOT NULL,
    enrolled_at TEXT NOT NULL,
    UNIQUE(course_uri, did)
);
CREATE INDEX IF NOT EXISTS idx_enrollments_course ON enrollments(course_uri);
CREATE INDEX IF NOT EXISTS idx_enrollments_did ON enrollments(did);
