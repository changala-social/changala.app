-- Add slot column to sessions so we can query by slot directly
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS slot TEXT;

-- Add slot preference to enrollments (NULL = see all sessions for the course)
ALTER TABLE enrollments ADD COLUMN IF NOT EXISTS slot TEXT;

-- Index for efficient filtering
CREATE INDEX IF NOT EXISTS idx_sessions_slot ON sessions(slot);
CREATE INDEX IF NOT EXISTS idx_enrollments_slot ON enrollments(slot);

-- Drop the old unique constraint and add a new one that includes slot
-- This allows a student to enroll in the same course with different slots (shouldn't happen, but defensive)
-- Actually keep UNIQUE(course_uri, did) — a student picks ONE slot per course
