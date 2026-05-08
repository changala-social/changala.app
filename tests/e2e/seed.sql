-- Changala E2E Test Seed Data
-- PostgreSQL seed for integration / end-to-end tests.

-- Seed a membership (so getRole works for seeded user)
INSERT INTO memberships (did, institution_did, institution_domain, role, verified_email, verified_at)
VALUES ('did:plc:seedadmin', 'did:web:nitc-ac-in', 'nitc.ac.in', 'admin', 'admin@nitc.ac.in', '2025-01-01T00:00:00Z')
ON CONFLICT DO NOTHING;

INSERT INTO memberships (did, institution_did, institution_domain, role, verified_email, verified_at)
VALUES ('did:plc:seedstudent', 'did:web:nitc-ac-in', 'nitc.ac.in', 'student', 'student@nitc.ac.in', '2025-01-01T00:00:00Z')
ON CONFLICT DO NOTHING;

-- Seed a course
INSERT INTO courses (uri, rkey, title, code, department, semester, visibility, created_by, description, created_at)
VALUES (
  'at://did:web:ring.changala.local/app.changala.course/seed001',
  'seed001', 'Introduction to Algorithms', 'CS301', 'Computer Science', 'Fall 2025',
  'institution', 'did:plc:seedadmin', 'Fundamental algorithms and data structures', '2025-01-01T00:00:00Z'
) ON CONFLICT (uri) DO NOTHING;

-- Seed an enrollment
INSERT INTO enrollments (course_uri, did, enrolled_at)
VALUES ('at://did:web:ring.changala.local/app.changala.course/seed001', 'did:plc:seedstudent', '2025-01-15T00:00:00Z')
ON CONFLICT DO NOTHING;

-- Seed a session (status=ended so keyword window tests work)
INSERT INTO sessions (uri, rkey, course_uri, scheduled_at, duration_mins, status, created_by, topic, opened_at, closed_at, keyword_window_expires_at, created_at)
VALUES (
  'at://did:web:ring.changala.local/app.changala.session/seed001',
  'seed001', 'at://did:web:ring.changala.local/app.changala.course/seed001',
  '2025-02-01T10:00:00Z', 60, 'ended', 'did:plc:seedadmin', 'Sorting Algorithms',
  '2025-02-01T10:00:00Z', '2025-02-01T11:00:00Z', '2025-02-01T12:00:00Z', '2025-01-20T00:00:00Z'
) ON CONFLICT (uri) DO NOTHING;

-- Seed keywords for the session
INSERT INTO keywords (session_uri, did, text, keyword_uri, created_at)
VALUES
  ('at://did:web:ring.changala.local/app.changala.session/seed001', 'did:plc:seedstudent', 'quicksort', '', '2025-02-01T10:30:00Z'),
  ('at://did:web:ring.changala.local/app.changala.session/seed001', 'did:plc:seedstudent', 'mergesort', '', '2025-02-01T10:31:00Z'),
  ('at://did:web:ring.changala.local/app.changala.session/seed001', 'did:plc:seedadmin', 'quicksort', '', '2025-02-01T10:32:00Z')
ON CONFLICT DO NOTHING;

-- Seed a note
INSERT INTO notes (uri, session_uri, author_did, format, ring_did, cid, version, summary, created_at)
VALUES (
  'at://did:plc:seedstudent/app.changala.note/seed001',
  'at://did:web:ring.changala.local/app.changala.session/seed001',
  'did:plc:seedstudent', 'latex', 'did:web:ring.changala.local', 'sha256-seednotecid001', 1,
  'Notes on sorting algorithm complexity', '2025-02-01T11:30:00Z'
) ON CONFLICT DO NOTHING;

-- Seed a vote on the note
INSERT INTO votes (vote_uri, subject_uri, voter_did, created_at)
VALUES ('at://did:plc:seedadmin/app.changala.vote/seed001', 'at://did:plc:seedstudent/app.changala.note/seed001', 'did:plc:seedadmin', '2025-02-02T00:00:00Z')
ON CONFLICT DO NOTHING;

-- Seed brain nodes
INSERT INTO brain_nodes (uri, author_did, title, format, ring_did, cid, tags, academic_ref, version, summary, created_at)
VALUES
  ('at://did:plc:brainuser/app.changala.brain.node/seed001', 'did:plc:brainuser', 'Divide and Conquer', 'markdown', 'did:web:ring.changala.local', 'sha256-brainnode001', '["algorithms","paradigms"]', 'at://did:web:ring.changala.local/app.changala.course/seed001', 1, 'Core algorithmic paradigm', '2025-02-05T00:00:00Z'),
  ('at://did:plc:brainuser/app.changala.brain.node/seed002', 'did:plc:brainuser', 'Recursion Patterns', 'markdown', 'did:web:ring.changala.local', 'sha256-brainnode002', '["recursion","patterns"]', NULL, 1, 'Common recursion patterns in CS', '2025-02-06T00:00:00Z')
ON CONFLICT DO NOTHING;

-- Seed a brain link
INSERT INTO brain_links (link_uri, from_uri, to_uri, label, created_by, created_at)
VALUES (
  'at://did:plc:brainuser/app.changala.brain.link/seed001',
  'at://did:plc:brainuser/app.changala.brain.node/seed002',
  'at://did:plc:brainuser/app.changala.brain.node/seed001',
  'relates to', 'did:plc:brainuser', '2025-02-06T01:00:00Z'
) ON CONFLICT DO NOTHING;

-- Seed a notification
INSERT INTO notifications (id, recipient_did, reason, subject_uri, read, created_at)
VALUES ('notif-seed-001', 'did:plc:seedstudent', 'vote_received', 'at://did:plc:seedstudent/app.changala.note/seed001', FALSE, '2025-02-02T00:00:00Z')
ON CONFLICT DO NOTHING;

-- Seed a sealed archive
INSERT INTO archives (archive_uri, course_uri, semester, sealed_by, ring_did, cid, session_count, note_count, status, initiated_at, sealed_at)
VALUES (
  'at://did:web:ring.changala.local/app.changala.archive/seed001',
  'at://did:web:ring.changala.local/app.changala.course/seed001',
  'Fall 2025', 'did:plc:seedadmin', 'did:web:ring.changala.local', 'sha256-archivecid001',
  1, 1, 'sealed', '2025-06-01T00:00:00Z', '2025-06-15T00:00:00Z'
) ON CONFLICT DO NOTHING;
