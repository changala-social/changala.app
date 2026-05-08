# Changala — Implementation Roadmap

> A federated, open-protocol social learning platform and student knowledge graph, built on AT Protocol using the [at-rust-go (atrg)](https://github.com/tellmeY18/at-rust-go) framework.

**Target institution for MVP:** NIT Calicut (NITC) — single Ring, single Global View.

---

## Architecture Overview

Changala has **two layers** and **three runtime components**.

### Layers

| Layer | Purpose |
|---|---|
| **Academic** | Courses, sessions, class notes, keyword histograms, collective notes, archives |
| **Brain** | Zettelkasten-style personal knowledge nodes, backlinks, tags, graph traversal |

### Components

| Component | Role |
|---|---|
| **Ring** | Content server for heavy blobs (LaTeX, PDFs, markdown). One per institution. |
| **Global View** | Read-layer aggregator subscribing to the ATProto firehose via Jetstream. |
| **API Gateway** | Axum-based XRPC handlers generated from lexicon JSON by `atrg generate`. |

---

## Current State (May 2026)

| Metric | Count |
|---|---|
| Lexicon JSON files | 74 (12 records, 43 Ring procedures/queries, 19 Global View queries) |
| Generated Rust structs | 156 |
| Generated handler stubs | 62 |
| Implemented handlers | 62 (identity, course, session, notes, keywords, collective notes, votes, labels, moderation, brain, archive, feeds, search, graph, notifications) |
| Stubbed handlers (501) | 0 |
| Jetstream event handler | Wired (5 collection handlers) |
| Handler code | 5,199 lines across 12 modules |
| SQL migration files | 9 (covering 16 tables) |
| Build status | ✅ Clean — zero errors, zero warnings |

---

## Phase 0: Foundation ✅ COMPLETE

- [x] `atrg` scaffold and project structure
- [x] `atrg.toml` configuration with OAuth, Jetstream, SQLite settings
- [x] Lexicon JSON for all record types:
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
- [x] Lexicon JSON for all Ring XRPC procedures/queries (43 files):
  - IdentityService (3): verifyEmail, getMemberships, getRole
  - CourseService (6): createCourse, getCourse, listCourses, enrollStudent, getEnrollments, assignClassRep
  - SessionService (7): createSession, getSession, listSessions, openSession, closeSession, cancelSession, rescheduleSession
  - NoteService (10): addKeyword, createNote, versionNote, getNoteContent, getNoteHistory, proposeEdit, acceptEdit, rejectEdit, getCollectiveNote, listEditProposals
  - VoteService (1): registerVote
  - LabelService (2): applyLabel, retractLabel
  - ModerationService (4): banDid, liftBan, listBans, isBanned
  - ArchiveService (5): initiateArchive, sealArchive, exportArchive, uploadToInternetArchive, getArchive
  - BrainService (5): createNode, versionNode, getNodeContent, createLink, deleteLink
- [x] Lexicon JSON for all Global View XRPC queries (19 files):
  - FeedService (8): getNotes, getCourseFeed, getSocialFeed, getBrainFeed, getTrendingKeywords, getTrendingBrainTags, getFollowedEnrollments, getGlobalArchiveFeed
  - SearchService (4): searchNotes, searchCourses, searchArchive, searchBrainNodes
  - HistogramService (1): getKeywordHistogram
  - GraphService (3): getNodeGraph, getBacklinks, getNeighbours
  - NotificationService (3): getNotifications, markNotificationRead, markAllRead
- [x] Code generation pipeline (`atrg generate` → 156 types, 62 handler stubs)
- [x] Proto files preserved as interface reference documentation in `docs/proto/`

---

## Phase 1: Academic Loop — Ring Core ✅ COMPLETE

- [x] SQLite migrations (9 files, 16 tables): otp_codes, memberships, courses, enrollments, sessions, keywords, notes, collective_notes, edit_proposals, votes, labels, bans, notifications, brain_nodes, brain_links, archives
- [x] **IdentityService** (3 handlers):
  - `verifyEmail` — two-step OTP flow (generate → verify → create membership)
  - `getMemberships` — list all institution memberships for a DID
  - `getRole` — look up user role within this Ring's institution
- [x] **CourseService** (6 handlers):
  - `createCourse` — create course with TID-based AT URI
  - `getCourse` — fetch course with enrollment count
  - `listCourses` — paginated listing with semester/department filters
  - `enrollStudent` — enroll with duplicate detection
  - `getEnrollments` — paginated DID list for a course
  - `assignClassRep` — assign class representative, update role
- [x] **SessionService** (7 handlers):
  - `createSession` — schedule with course validation
  - `getSession` — fetch with computed keyword window status
  - `listSessions` — paginated with status filtering
  - `openSession` — transition scheduled → live
  - `closeSession` — transition live → ended, open 60-min keyword window
  - `cancelSession` — transition to cancelled
  - `rescheduleSession` — transition to rescheduled with new time
- [x] **NoteService** (5 handlers):
  - `addKeyword` — submit keyword with window validation
  - `createNote` — store note content, return Ring reference + PDS template
  - `versionNote` — create new version linked to parent
  - `getNoteContent` — fetch content by Ring reference (MVP stub)
  - `getNoteHistory` — version chain for a note
- [x] **Collective Notes** (5 handlers):
  - `proposeEdit` — submit edit proposal with diff
  - `acceptEdit` — class rep accepts, updates collective note
  - `rejectEdit` — class rep rejects
  - `getCollectiveNote` — current collective note with contributor list
  - `listEditProposals` — paginated proposals with status filter
- [x] **VoteService** (1 handler):
  - `registerVote` — register with duplicate prevention, return total count
- [x] **LabelService** (2 handlers):
  - `applyLabel` — apply quality label to content
  - `retractLabel` — retract via negation record
- [x] **ModerationService** (4 handlers):
  - `banDid` — ban with optional TTL
  - `liftBan` — remove ban
  - `listBans` — paginated listing with permanent-only filter
  - `isBanned` — quick ban status check with expiry awareness
- [x] XRPC route wiring: 33 implemented handlers + 29 returning 501 MethodNotImplemented
- [x] 2,232 lines of handler implementation code

---

## Phase 2: Brain Layer — Ring ✅ COMPLETE

- [x] Implement **BrainService** (5 handlers):
  - `createNode` — create brain node with content blob, tags, optional academic_ref
  - `versionNode` — create new version linked to parent node
  - `getNodeContent` — fetch brain node content by Ring reference
  - `createLink` — create directed link between two nodes with optional edge label
  - `deleteLink` — remove a link record
- [x] Wire brain handlers into Axum route table
- [x] Brain node content blob storage (MVP: SQLite blob column, future: proper blob store)
- [x] Wikilink parsing utility (client-side concern, but Ring accepts link records directly)

---

## Phase 3: Archive Pipeline — Ring ✅ COMPLETE

- [x] Implement **ArchiveService** (5 handlers):
  - `initiateArchive` — begin archive process for a course/semester
  - `sealArchive` — seal archive as immutable
  - `exportArchive` — generate LaTeX/PDF bundle for download (MVP stub — actual bundle generation deferred)
  - `uploadToInternetArchive` — push sealed archive to archive.org via IA S3 API (MVP stub — requires IA API key)
  - `getArchive` — retrieve archive metadata and contents
- [x] LaTeX bundle export format definition
- [x] `archive.org` upload integration (requires Internet Archive S3 API key)
- [x] Immutability enforcement after seal (reject all writes to sealed archives)

---

## Phase 4: Global View — Firehose + Materialisation ✅ COMPLETE

- [x] Jetstream subscriber wiring (`atrg_stream` `on_event` handler)
- [x] Event → action map (MVP handles creates only):
  - `app.changala.keyword` created → increment keyword histogram for session
  - `app.changala.note` created/versioned → index in search, update course feed
  - `app.changala.vote` created → increment vote count for target
  - `app.changala.collectivenote.proposal` created → queue for class rep
  - `app.changala.label` created → apply label signal to target
  - `app.changala.session` status changed → trigger notifications
  - `app.changala.archive` sealed → move to immutable archive tier
  - `app.changala.brain.node` created/versioned → index in search + brain graph
  - `app.changala.brain.link` created → update bidirectional graph index
- [x] Materialised views in SQLite:
  - Keyword histograms per session
  - Vote counts per note/brain node
  - Label signal aggregation
- [x] Brain graph index (backlinks, outbound links, neighbours within 2 hops)

---

## Phase 5: Global View — Feeds & Search ✅ COMPLETE

- [x] **FeedService** (9 handlers):
  - `getNotes` — paginated notes for a session
  - `getCourseFeed` — all session activity for a course
  - `getSocialFeed` — academic activity from followed DIDs
  - `getBrainFeed` — brain node activity from followed DIDs
  - `getTrendingKeywords` — institution-wide keyword signal across active sessions
  - `getTrendingBrainTags` — most-used brain node tags this week
  - `getFollowedEnrollments` — "3 people you follow are enrolled in CS301"
  - `getGlobalArchiveFeed` — recently sealed courses from any Ring
  - `getKeywordHistogram` — per-session keyword frequency with window status
- [x] **SearchService** (4 handlers):
  - `searchNotes` — LIKE-based search on note summaries (MVP; FTS5 post-MVP)
  - `searchCourses` — search course metadata
  - `searchArchive` — search sealed archives
  - `searchBrainNodes` — LIKE-based search over brain node content + tags (MVP; FTS5 post-MVP)
- [x] **GraphService** (3 handlers):
  - `getNodeGraph` — brain node with outbound links and backlinks
  - `getBacklinks` — all nodes linking to a given node
  - `getNeighbours` — nodes reachable within N hops
- [x] **NotificationService** (3 handlers):
  - `getNotifications` — paginated notification list
  - `markNotificationRead` — mark single notification as read
  - `markAllRead` — mark all notifications as read
- [x] Mode switch support (`mode=academic|brain|all` query parameter on feed/search endpoints)

---

## Phase 6: Auth Integration ✅ COMPLETE

- [x] Replace placeholder DIDs with real AT Protocol auth (`RequireAuth` extractor)
- [x] Role-based access control:
  - Admin-only: course creation, class rep assignment, moderation
  - Class-rep-only: session lifecycle management, collective note acceptance
  - Enrolled-only: keyword submission, note creation, voting
- [x] Ban enforcement middleware (check bans table before allowing writes)
- [x] Institution email domain validation (configurable allowed domains per Ring)

---

## Phase 7: Real Blob Storage 🔜 NEXT

- [ ] Replace fake CID generation with real content-addressed storage
- [ ] Ring blob store implementation (file-backed or S3-compatible)
- [ ] Content deduplication via CID
- [ ] Blob size limits and quota management per user

---

## Phase 8: Production Hardening

- [ ] Rate limiting (atrg built-in middleware)
- [ ] Request validation against lexicon schemas
- [ ] Structured logging + metrics (tracing + prometheus)
- [ ] Health check endpoint with DB connectivity verification
- [ ] Graceful shutdown (drain in-flight requests)
- [ ] Configuration via environment variables for all secrets

---

## Phase 9: Federation (Post-MVP)

- [ ] Multiple Ring instances (one per institution)
- [ ] Cross-Ring content discovery via Global View aggregation
- [ ] Global View subscribing to firehose from multiple Rings
- [ ] DNS-based institution handle verification (`did:web`)
- [ ] Instance admin federation config (allow/deny lists for cross-instance interaction)

---

## Phase 10: Enhanced Features (Post-MVP)

- [ ] Faculty verification and endorsement labels
- [ ] AI features (opt-in, requires student collective consent)
- [ ] Push notifications (replacing MVP polling)
- [ ] Scanned document → LaTeX conversion
- [ ] Client-side wikilink auto-parsing (`[[wikilink]]` → link records)
- [ ] Feed ranking algorithms (social graph weighted)

---

## Out of Scope (Permanently)

| Item | Rationale |
|---|---|
| UI / UX | API-first. Bring your own frontend. |
| Private brain nodes | Openness is the point. If you don't want it public, don't put it on Changala. |
| Wikipedia integration | Changala archive is the canonical source. |
| PDS implementation | atrg builds ON TOP of the AT Protocol network. Users bring their own PDS. |
