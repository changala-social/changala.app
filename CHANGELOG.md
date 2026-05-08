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

### Changed
- Migrated from gRPC/protobuf architecture to AT Protocol XRPC/lexicon architecture
- Replaced proto-based code generation with `atrg generate` lexicon-based generation
- Moved proto files from project root to `docs/proto/` (reference documentation only)
- Removed buf.yaml and buf.gen.yaml (protobuf toolchain no longer used)
- Updated Cargo.toml with full atrg dependency set (atrg-xrpc, atrg-stream, atrg-repo, atrg-identity, sqlx, chrono)
- Updated atrg.toml with Jetstream collection subscriptions for all app.changala.* record types
- Renamed `type` property to `reason`/`eventType` in 4 lexicons to avoid Rust keyword conflict in generated code
