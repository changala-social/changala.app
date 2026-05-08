# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

#### Foundation (Phase 0)
- Project scaffold using at-rust-go (atrg) framework
- `atrg.toml` configuration with OAuth, Jetstream, SQLite settings
- Lexicon JSON files for all AT Protocol record types:
  - `app.changala.defs` — shared types (RingRef, Role, SessionStatus, NoteFormat, Visibility, LabelVal, NotificationType, LabelSignal, Notification)
  - `app.changala.membership` — institution affiliation credential
  - `app.changala.course` — course record (Ring DID repo)
  - `app.changala.session` — class session record
  - `app.changala.keyword` — student keyword submission
  - `app.changala.note` — note metadata + Ring reference
  - `app.changala.vote` — upvote record
  - `app.changala.label` — knowledge quality label
  - `app.changala.archive` — sealed semester archive
  - `app.changala.collectivenote.proposal` — collective note edit proposal
  - `app.changala.brain.node` — Zettelkasten knowledge node
  - `app.changala.brain.link` — directed link between nodes
- 43 Ring XRPC procedure/query lexicons covering: IdentityService (3), CourseService (6), SessionService (7), NoteService (10), VoteService (1), LabelService (2), ModerationService (4), ArchiveService (5), BrainService (5)
- 19 Global View XRPC query lexicons covering: FeedService (8), SearchService (4), HistogramService (1), GraphService (3), NotificationService (3)
- Code generation via `atrg generate` producing 156 typed Rust structs and 62 Axum handler stubs
- Proto files preserved as interface reference documentation in `docs/proto/`

#### Academic Loop — Ring Core (Phase 1)
- SQLite migrations (9 files, 16 tables): otp_codes, memberships, courses, enrollments, sessions, keywords, notes, collective_notes, edit_proposals, votes, labels, bans, notifications, brain_nodes, brain_links, archives
- **IdentityService** (3 handlers):
  - `verifyEmail` — two-step OTP flow (generate → verify → create membership)
  - `getMemberships` — list all institution memberships for a DID
  - `getRole` — look up user role within this Ring's institution
- **CourseService** (6 handlers):
  - `createCourse` — create course with TID-based AT URI
  - `getCourse` — fetch course with enrollment count
  - `listCourses` — paginated listing with semester/department filters
  - `enrollStudent` — enroll with duplicate detection
  - `getEnrollments` — paginated DID list for a course
  - `assignClassRep` — assign class representative, update role
- **SessionService** (7 handlers):
  - `createSession` — schedule with course validation
  - `getSession` — fetch with computed keyword window status
  - `listSessions` — paginated with status filtering
  - `openSession` — transition scheduled → live
  - `closeSession` — transition live → ended, open 60-min keyword window
  - `cancelSession` — transition to cancelled
  - `rescheduleSession` — transition to rescheduled with new time
- **NoteService** (5 handlers):
  - `addKeyword` — submit keyword with window validation
  - `createNote` — store note content, return Ring reference + PDS template
  - `versionNote` — create new version linked to parent
  - `getNoteContent` — fetch content by Ring reference (MVP stub)
  - `getNoteHistory` — version chain for a note
- **Collective Notes** (5 handlers):
  - `proposeEdit` — submit edit proposal with diff
  - `acceptEdit` — class rep accepts, updates collective note
  - `rejectEdit` — class rep rejects
  - `getCollectiveNote` — current collective note with contributor list
  - `listEditProposals` — paginated proposals with status filter
- **VoteService** (1 handler):
  - `registerVote` — register with duplicate prevention, return total count
- **LabelService** (2 handlers):
  - `applyLabel` — apply quality label to content
  - `retractLabel` — retract via negation record
- **ModerationService** (4 handlers):
  - `banDid` — ban with optional TTL
  - `liftBan` — remove ban
  - `listBans` — paginated listing with permanent-only filter
  - `isBanned` — quick ban status check with expiry awareness
- XRPC route wiring: 33 implemented handlers + 29 returning 501 MethodNotImplemented
- 2,232 lines of handler implementation code

#### Brain Layer — Ring (Phase 2)
- **BrainService** (5 handlers):
  - `createNode` — create brain node with tags, optional academic_ref, fake CID for MVP
  - `versionNode` — create new version linked to parent with inherited author
  - `getNodeContent` — fetch brain node content by Ring reference (MVP stub)
  - `createLink` — create directed link between nodes with optional edge label
  - `deleteLink` — remove a link record

#### Archive Pipeline — Ring (Phase 3)
- **ArchiveService** (5 handlers):
  - `initiateArchive` — begin archive for course/semester, count sessions and notes
  - `sealArchive` — seal archive as immutable with generated bundle CID
  - `exportArchive` — generate export reference (MVP stub — actual bundle generation deferred)
  - `uploadToInternetArchive` — mock Internet Archive upload (MVP stub — requires IA API key)
  - `getArchive` — retrieve archive metadata by course URI and semester

#### Firehose Materialisation (Phase 4)
- Jetstream event handler wired into `AtrgApp::on_event()`
- Event handlers for 5 collections:
  - `app.changala.keyword` → INSERT OR IGNORE into keywords table
  - `app.changala.vote` → deduplicated vote materialisation
  - `app.changala.label` → label assertion/retraction storage
  - `app.changala.brain.node` → INSERT OR REPLACE for versioned brain nodes
  - `app.changala.brain.link` → deduplicated link materialisation
- Jetstream consumer connects to `jetstream1.us-east.bsky.network` subscribing to all `app.changala.*` collections

#### Global View — Feeds, Search, Graph, Notifications (Phase 5)
- **FeedService** (9 handlers):
  - `getNotes` — session notes with vote counts and labels, sorted by votes or date
  - `getCourseFeed` — combined session events + note activity for a course
  - `getSocialFeed` — recent activity with mode switch (academic/brain/all)
  - `getBrainFeed` — brain nodes with vote counts, parsed tags
  - `getTrendingKeywords` — keyword aggregation within configurable time window
  - `getTrendingBrainTags` — in-memory tag counting from recent brain nodes
  - `getFollowedEnrollments` — MVP: most popular courses by enrollment count
  - `getGlobalArchiveFeed` — sealed archives with course metadata
  - `getKeywordHistogram` — per-session keyword frequency with window status
- **SearchService** (4 handlers):
  - `searchNotes` — LIKE-based search on note summaries with course/semester filters
  - `searchCourses` — search course title and code with enrollment counts
  - `searchArchive` — search sealed archives via course metadata
  - `searchBrainNodes` — search brain node title/summary with tag and author filters
- **GraphService** (3 handlers):
  - `getNodeGraph` — BFS-based local subgraph traversal (configurable depth 1-3, max 100 nodes)
  - `getBacklinks` — paginated list of nodes linking to a given node
  - `getNeighbours` — flat neighbour list with distance, BFS-based
- **NotificationService** (3 handlers):
  - `getNotifications` — paginated with unread-only filter and unread count
  - `markNotificationRead` — mark single notification as read
  - `markAllRead` — bulk mark all as read
- All XRPC routes wired: 62 implemented handlers, 0 stubs remaining
- Total handler code: 5,199 lines across 12 modules

#### Auth Integration (Phase 6)
- Shared auth helper module (`handlers/auth.rs`) with 6 functions:
  - `check_not_banned` — ban enforcement on all write operations
  - `get_role` / `require_role` — role-level hierarchy (admin > classRep > student)
  - `require_enrolled` — course enrollment verification
  - `require_class_rep_or_admin` — session lifecycle authorization
  - `get_course_for_session` — session-to-course URI resolution
- `RequireAuth` extractor added to 31 handler functions across 7 modules:
  - `course.rs` — createCourse (admin), enrollStudent, assignClassRep (admin)
  - `session.rs` — createSession, openSession, closeSession, cancelSession, rescheduleSession (all class-rep-or-admin)
  - `note.rs` — addKeyword, createNote, versionNote, proposeEdit, acceptEdit (class-rep), rejectEdit (class-rep), registerVote, applyLabel, retractLabel
  - `brain.rs` — createNode, versionNode, createLink, deleteLink
  - `archive.rs` — initiateArchive (admin), sealArchive (admin), exportArchive, uploadToInternetArchive (admin)
  - `moderation.rs` — banDid (admin), liftBan (admin), listBans (admin)
  - `notification.rs` — getNotifications, markNotificationRead, markAllRead (scoped to authenticated user's DID)
- Removed all `PLACEHOLDER_DID` constants — every write operation now uses the authenticated user's DID
- Public read endpoints remain unauthenticated: listCourses, getCourse, getSession, listSessions, getNoteContent, getNoteHistory, getCollectiveNote, listEditProposals, getEnrollments, isBanned, getArchive, all search/feed/graph endpoints
- Ban enforcement: all write handlers check `bans` table before processing
- Role-based access control:
  - Admin-only: course creation, class rep assignment, moderation, archive initiation/sealing/IA upload
  - Class-rep-or-admin: session lifecycle management, collective note acceptance/rejection
  - Any authenticated: enrollment, note/keyword/vote/label/brain node operations

#### Real Blob Storage + PostgreSQL Migration (Phases 7–8)
- **PostgreSQL migration**: all 16 business tables moved from SQLite to PostgreSQL
  - Separate migration directory `pg_migrations/` (9 files) for Postgres schema
  - atrg internal tables (sessions, OAuth states) remain in SQLite under `migrations/`
  - SQL dialect changes: `?` → `$1,$2,...` placeholders, `AUTOINCREMENT` → `BIGSERIAL`, `datetime('now')` → chrono-computed timestamps, `INSERT OR IGNORE` → `ON CONFLICT DO NOTHING`, SQLite boolean integers → Postgres `BOOLEAN`
- **S3 blob store** (`src/blob.rs`):
  - Content-addressed storage using SHA-256 CIDs (`sha256-{hex}` format)
  - `put(data)` → stores blob, returns CID
  - `get(cid)` → retrieves blob bytes
  - `exists(cid)` / `delete(cid)` for lifecycle management
  - Compatible with any S3 service (RustFS, MinIO, AWS S3) via `rust-s3` crate
  - Configurable endpoint, bucket, region, credentials via `[changala.s3]` in atrg.toml
- **Global application state** (`src/state.rs`):
  - `Changala` struct holding `PgPool` + `Arc<S3BlobStore>`
  - `OnceCell`-based global state pattern — initialized once at startup, accessible from all handlers and the Jetstream event processor without Axum state extractor plumbing
- **Handler migration** — all 62 handlers updated:
  - `State(state): State<AppState>` removed from all handler params
  - `state.db` (SQLite) replaced with `crate::state::get().db` (Postgres)
  - `fake_cid()` replaced with real `app.blobs.put()` in note, brain, and archive handlers
  - `get_note_content` and `get_node_content` now fetch real content from S3 instead of returning stubs
  - All auth helper functions (`auth.rs`) now take `&PgPool` directly
- **Configuration** (`atrg.toml`):
  - Added `[changala]` section with `database_url` for PostgreSQL connection
  - Added `[changala.s3]` section with endpoint, bucket, region, credentials, path_style
- **Dependencies added**: `rust-s3`, `sha2`, `hex`, `once_cell`, `toml`; sqlx now has both `sqlite` and `postgres` features

### Changed
- Migrated from gRPC/protobuf architecture to AT Protocol XRPC/lexicon architecture
- Replaced proto-based code generation with `atrg generate` lexicon-based generation
- Moved proto files from project root to `docs/proto/` (reference documentation only)
- Removed buf.yaml and buf.gen.yaml (protobuf toolchain no longer used)
- Updated Cargo.toml with full atrg dependency set (atrg-xrpc, atrg-stream, atrg-repo, atrg-identity, sqlx, chrono)
- Updated atrg.toml with Jetstream collection subscriptions for all app.changala.* record types
- Renamed `type` property to `reason`/`eventType` in 4 lexicons to avoid Rust keyword conflict in generated code
