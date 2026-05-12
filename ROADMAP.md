# Changala — Implementation Roadmap

> A federated, open-protocol social learning platform and student knowledge graph, built on AT Protocol using the [at-rust-go (atrg)](https://github.com/tellmeY18/at-rust-go) framework.

**Target institution for MVP:** NIT Calicut (NITC) — single Ring, single Global View.

---

## Architecture Overview

Changala has **two layers**, **three runtime components**, and ships as
**two binaries** from one monorepo.

### Layers

| Layer | Purpose |
|---|---|
| **Academic** | Courses, sessions, class notes, keyword histograms, collective notes, archives |
| **Brain** | Zettelkasten-style personal knowledge nodes, backlinks, tags, graph traversal |

### Components

| Component | Binary | Deployed by | Role |
|---|---|---|---|
| **Ring** | `changala-ring` | Each institution | Write server: identity, OAuth, courses, sessions, notes, brain CRUD, blob storage, moderation, archives |
| **Aggregator** | `changala-aggregator` | `changala.app` (canonical) | Read-only: firehose subscriber, feeds, search, graph index, notifications, cross-institution aggregation |
| **Frontend** | Static (CF Pages) | `changala-app.pages.dev` | React SPA. Reads from Aggregator, writes to Ring. No per-institution UI — shared frontend for all. |

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
| Build status | ✅ Clean — zero errors, zero warnings (Postgres + S3) |
| Database | PostgreSQL (via CNPG) |
| Blob storage | S3-compatible (RustFS) |

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

## Phase 7: Real Blob Storage ✅ COMPLETE

- [x] Replace fake CID generation with real content-addressed storage
- [x] Ring blob store implementation (file-backed or S3-compatible)
- [x] Content deduplication via CID
- [x] Blob size limits and quota management per user

---

## Phase 8: Production Hardening ✅ COMPLETE

- [x] Rate limiting (atrg built-in middleware)
- [x] Request validation against lexicon schemas
- [x] Structured logging + metrics (tracing + prometheus)
- [x] Health check endpoint with DB connectivity verification
- [x] Graceful shutdown (drain in-flight requests)
- [x] Configuration via environment variables for all secrets

---

## Phase 8.5: Identity, Email Verification & RBAC 🔜 NEXT (simplified by atrg 0.2.0)

> **Prerequisite: Complete Phase 8.6 (atrg 0.2.0 migration) first.**
> Most infrastructure work in this phase (SMTP, OTP, RBAC, admin provisioning)
> is now provided by `atrg-email` and `atrg-auth`. This phase reduces to
> wiring the framework modules + Changala-specific business logic only.

> User identity tiers, institution email verification with domain allowlists,
> SMTP-based OTP delivery (Gmail app password), and admin provisioning.

### 8.5.1 User Identity Tiers

Changala has **three identity tiers**. AT Protocol login alone does NOT grant
access to institution-specific features — it only proves you control a DID.

| Tier | Who | How obtained | Can do |
|---|---|---|---|
| **Public viewer** | Anyone (no login) | Visit the site | Read all world-public content: courses, sessions, notes, brain nodes, archives, search, graph |
| **AT Proto user** | Logged in via AT Protocol OAuth | Click "Login" | Everything public viewers can do, plus: create brain nodes, create links, vote on public content, view notifications |
| **Institution member** | AT Proto user + verified institution email | Verify email via OTP | Everything AT Proto users can do, plus: enroll in courses, submit keywords, create notes, propose edits. Role (student/classRep/admin) determines further permissions within the institution. |

**Key distinction:** An AT Protocol login is NOT an institution membership.
A Bluesky user who logs in can browse and interact with the brain layer (public),
but cannot enroll in courses or participate in academic sessions until they
verify an institution email.

### 8.5.2 Email Domain Allowlist

Each Ring instance configures a list of allowed email domains. The `verifyEmail`
endpoint rejects any email address whose domain is not in the allowlist.
No Gmail, Yahoo, or other consumer email providers — only institution domains.

**Configuration:**

```toml
# atrg.toml
[changala]
allowed_email_domains = ["nitc.ac.in", "mbcet.ac.in"]
```

**Env var override:**

```
CHANGALA_ALLOWED_EMAIL_DOMAINS=nitc.ac.in,mbcet.ac.in
```

**Validation logic** (in `verifyEmail` handler):

```
1. Extract domain from email: "student@nitc.ac.in" → "nitc.ac.in"
2. Check domain against allowlist
3. If not in allowlist → 400 InvalidRequest: "Email domain not allowed. Use your institution email."
4. If in allowlist → proceed with OTP generation
```

### 8.5.3 SMTP Email Delivery (Gmail App Password)

Replace the current OTP-logging-to-stdout with real email delivery via SMTP.
Use Gmail with a 2FA app password — simplest option, no third-party API needed.

**Approach** (modelled after [Apache Answer](https://github.com/apache/answer)'s
email service which uses `gomail` with SMTP host/port/user/pass/encryption):

- Rust crate: [`lettre`](https://crates.io/crates/lettre) — mature, async,
  supports STARTTLS + TLS + plain, handles Gmail app passwords.
- Connection: STARTTLS on port 587 (Gmail default for app passwords).
- Authentication: LOGIN with Gmail address + 16-char app password.

**Configuration:**

```toml
# atrg.toml
[changala.smtp]
host = "smtp.gmail.com"
port = 587
username = "changala.ring@gmail.com"
password = "xxxx xxxx xxxx xxxx"   # Gmail 2FA app password
from = "Changala <changala.ring@gmail.com>"
encryption = "starttls"             # "starttls" | "tls" | "none"
```

**Env var overrides (for k8s Secrets):**

| Env Var | Config Path | Example |
|---|---|---|
| `CHANGALA_SMTP_HOST` | `[changala.smtp] host` | `smtp.gmail.com` |
| `CHANGALA_SMTP_PORT` | `[changala.smtp] port` | `587` |
| `CHANGALA_SMTP_USERNAME` | `[changala.smtp] username` | `changala.ring@gmail.com` |
| `CHANGALA_SMTP_PASSWORD` | `[changala.smtp] password` | `xxxx xxxx xxxx xxxx` |
| `CHANGALA_SMTP_FROM` | `[changala.smtp] from` | `Changala <changala.ring@gmail.com>` |
| `CHANGALA_SMTP_ENCRYPTION` | `[changala.smtp] encryption` | `starttls` |

**Gmail setup steps:**

1. Create a Gmail account for the Ring (e.g. `changala.nitc@gmail.com`)
2. Enable 2-Factor Authentication on the account
3. Generate an App Password: Google Account → Security → App Passwords
4. Use the 16-character app password as `CHANGALA_SMTP_PASSWORD`
5. Set `CHANGALA_SMTP_USERNAME` to the Gmail address

**OTP email template:**

```
Subject: Changala — Your verification code

Your verification code is: 482910

This code expires in 10 minutes.
If you did not request this, ignore this email.
```

**Fallback:** If SMTP is not configured (empty host), OTPs continue to be
logged to stdout (development mode).

### 8.5.4 Admin Provisioning

- [ ] `CHANGALA_ADMIN_DIDS` env var — comma-separated list of DIDs to auto-provision
      as admin on startup. Checked on boot, inserted into `memberships` with role `admin`.
- [ ] `POST /xrpc/app.changala.ring.provisionAdmin` — admin provisioning via
      shared secret (`CHANGALA_ADMIN_SECRET` env var). Allows bootstrapping the
      first admin without DB access.

### 8.5.5 Role Management Endpoints

Support for MCP

- [ ] `POST /xrpc/app.changala.ring.promoteRole` — admin endpoint to promote a user
- [ ] `POST /xrpc/app.changala.ring.demoteRole` — admin endpoint to demote a user
- [ ] Admin audit log table + `GET /xrpc/app.changala.ring.getAuditLog`

### 8.5.6 Implementation Checklist

**Migrations:**

- [ ] `0010_email_domains.sql` — add `allowed_email_domains` config table (or use atrg.toml)
- [ ] `0011_audit_log.sql` — `audit_log` table (actor_did, action, target_did, details, created_at)
- [ ] `0012_membership_tracking.sql` — add `promoted_by`, `promoted_at` to memberships

**Backend:**

- [ ] Add `lettre` dependency to `Cargo.toml`
- [ ] `src/email.rs` — SMTP email sender (connect, build message, send)
- [ ] `SmtpConfig` struct in `main.rs` with env var overrides
- [ ] Update `verifyEmail` handler: validate domain against allowlist, send OTP via SMTP
- [ ] `CHANGALA_ALLOWED_EMAIL_DOMAINS` env var + `apply_env_overrides()`
- [ ] `CHANGALA_SMTP_*` env vars + `apply_env_overrides()`
- [ ] `CHANGALA_ADMIN_DIDS` startup provisioning logic
- [ ] `CHANGALA_ADMIN_SECRET` + `provisionAdmin` endpoint
- [ ] `promoteRole` / `demoteRole` handlers
- [ ] Audit log writes on: role changes, bans, course creation, class rep assignment
- [ ] Update `enrollStudent` to require institution membership (not just RequireAuth)
- [ ] Update `addKeyword`, `createNote`, `proposeEdit` to require institution membership

**Frontend:**

- [ ] Email verification flow on `/dashboard` (prompt if logged in but no membership)
- [ ] Role management UI in admin panel
- [ ] Show membership status on profile page
- [ ] Distinguish "logged in but not verified" vs "verified member" in UI

**Ref:** See `RBAC.md` for the full permission matrix.

---

## Phase 8.6: atrg 0.2.0 Migration 🔥 CRITICAL — DO FIRST

> **This phase should be completed BEFORE Phase 8.5 and Phase 9.**
> atrg 0.2.0 provides native implementations of most features planned in Phase 8.5
> (email/OTP, RBAC, API keys, admin bootstrap) and Phase 9 (migration isolation,
> multi-binary template). Migrating first avoids building on deprecated patterns.

### Why Migrate Now

atrg 0.2.0 (released 2026-05-12) adds 11 features that directly replace hand-written
Changala code. Continuing to build on v0.1.x means maintaining ~800 lines of workarounds
that the framework now handles natively. Every new feature built on v0.1.x patterns
increases migration cost.

**Crates published in v0.2.0:**

| Crate | Version | Status |
|---|---|---|
| `atrg-core` | 0.2.0 | Breaking: `AppState` gains `extensions` field, `AppConfig` gains `admin_dids` |
| `atrg-auth` | 0.2.0 | Breaking: `AuthSource::ApiKey` variant added. New: API keys, RBAC, bans |
| `atrg-db` | 0.2.0 | Breaking: `run_user_migrations` deprecated → `run_isolated_migrations` |
| `atrg-stream` | 0.2.0 | Breaking: `StreamConfig` gains `cursor` field. New: `EventRouterBuilder`, cursor persistence |
| `atrg-blob` | 0.2.0 | **NEW** — `BlobStore` trait, `S3BlobStore`, `FileBlobStore`, `compute_cid()` |
| `atrg-email` | 0.2.0 | **NEW** — SMTP via lettre, `send_otp`/`verify_otp`, domain validation |
| `atrg-xrpc` | 0.2.0 | No breaking changes |
| `atrg-identity` | 0.2.0 | No breaking changes |
| `atrg-repo` | 0.2.0 | No breaking changes |
| `atrg-codegen` | 0.2.0 | No breaking changes |
| `atrg-feed` | 0.2.0 | No breaking changes |
| `atrg-label` | 0.2.0 | No breaking changes |
| `atrg-testing` | 0.2.0 | Updated for new `AppState` shape |
| `atrg-firehose` | 0.2.0 | No breaking changes |
| `atrg-cli` | 0.2.0 | New: `--template multi-binary` |

### 8.6.1 Code to Delete / Replace

| Current Code | Lines | Replacement | atrg Feature |
|---|---|---|---|
| `crates/changala-ring/src/state.rs` (OnceCell singleton) | 44 | `AtrgApp::with_extension::<Changala>()` | AppState Extensions |
| `crates/changala-aggregator/src/state.rs` (OnceCell singleton) | 27 | `AtrgApp::with_extension::<Aggregator>()` | AppState Extensions |
| `~40 call sites` of `crate::state::get()` across all handlers | pervasive | `state.extension::<T>()` via Axum `State` extractor | AppState Extensions |
| `crates/changala-ring/src/blob.rs` (S3BlobStore + compute_cid) | 97 | `atrg_blob::S3BlobStore` | atrg-blob |
| `crates/changala-ring/src/email.rs` (SMTP + OTP) | 78 | `atrg_email::send_otp()` / `verify_otp()` | atrg-email |
| `crates/changala-ring/src/api_key_auth.rs` (middleware bridge) | 153 | Native `RequireAuth` API key support | atrg-auth API Keys |
| `Ring main.rs: run_changala_migrations()` | ~40 | `atrg_db::run_isolated_migrations(pool, dir, "_ring_migrations")` | Migration Isolation |
| `Aggregator main.rs: run_aggregator_migrations()` | ~40 | `atrg_db::run_isolated_migrations(pool, dir, "_aggregator_migrations")` | Migration Isolation |
| `Ring main.rs: ChangalaConfig::apply_env_overrides()` | ~80 | `atrg_core::config::load_app_config::<ChangalaConfig>("changala")` | App-Specific Config |
| `Ring main.rs: admin DID provisioning` | ~20 | `[app] admin_dids` config + `ATRG_APP__ADMIN_DIDS` env var | Admin Bootstrap |
| `Ring main.rs: bootstrap API key provisioning` | ~30 | `atrg_auth::api_keys::create_api_key()` at startup | atrg-auth API Keys |
| `handlers/auth.rs` (hand-rolled RBAC) | 172 | `atrg_auth::rbac::has_role()`, `grant_role()`, `ban_did()` | RBAC |
| `handlers/apikeys.rs` (hand-rolled API key CRUD) | 216 | `atrg_auth::api_keys::create_api_key()`, `list_api_keys()`, `revoke_api_key()` | atrg-auth API Keys |
| Aggregator `events.rs` match-dispatch boilerplate | ~50 | `EventRouterBuilder::new().on_create("app.changala.keyword", handler)` | Event Router |
| Custom `/auth/complete` cross-origin endpoint | ~60 | `[auth] post_login_redirect = "https://changala-app.pages.dev/login"` | Cross-Origin Auth |
| **Total** | **~800+** | — | — |

### 8.6.2 Migration Steps (Ordered)

These steps must be done in order — each builds on the previous.

#### Step 1: Bump atrg dependencies

- [ ] Update `Cargo.toml` workspace dependencies from `0.1.3` to `0.2.0`
- [ ] Add new dependencies: `atrg-blob = "0.2.0"`, `atrg-email = "0.2.0"`
- [ ] Remove `once_cell` from workspace dependencies
- [ ] Remove `lettre` from workspace dependencies (now in `atrg-email`)
- [ ] Remove `sha2` and `hex` from workspace dependencies (now in `atrg-blob`)
- [ ] Run `cargo build` — expect compilation errors (this is the starting point)

#### Step 2: Fix AppState breaking changes

- [ ] Add `extensions: Arc::new(Extensions::new())` to any direct `AppState` construction (tests)
- [ ] Add `admin_dids: vec![]` to any direct `AppConfig` construction (tests)
- [ ] Add `cursor: None` to any direct `StreamConfig` construction
- [ ] Add `AuthSource::ApiKey` arm to any exhaustive matches on `AuthSource`

#### Step 3: Replace state management (`once_cell` → Extensions)

- [ ] Define `ChangalaState` struct (replaces `Changala` in `state.rs`):
  ```
  struct ChangalaState {
      db: PgPool,
      blobs: Arc<atrg_blob::S3BlobStore>,
      smtp: Option<atrg_email::SmtpConfig>,
      allowed_email_domains: Vec<String>,
  }
  ```
- [ ] Register in Ring `main.rs`: `AtrgApp::new().with_extension(changala_state)`
- [ ] Define `AggregatorState` struct (replaces `Aggregator` in `state.rs`):
  ```
  struct AggregatorState { db: PgPool }
  ```
- [ ] Register in Aggregator `main.rs`: `AtrgApp::new().with_extension(aggregator_state)`
- [ ] Replace all `crate::state::get()` calls with `state.extension::<ChangalaState>()`
  (or `state.extension::<AggregatorState>()` in aggregator handlers)
- [ ] Delete `crates/changala-ring/src/state.rs`
- [ ] Delete `crates/changala-aggregator/src/state.rs`
- [ ] Remove `once_cell` from all `Cargo.toml` files

#### Step 4: Replace blob storage

- [ ] Replace `crate::blob::S3BlobStore` with `atrg_blob::S3BlobStore`
- [ ] Replace `crate::blob::compute_cid()` with `atrg_blob::compute_cid()`
- [ ] Update `ChangalaState` to use `atrg_blob::S3BlobStore`
- [ ] Verify CID format compatibility (`sha256-` prefix — both use the same scheme)
- [ ] Update `brain.rs` and `note.rs` handler imports
- [ ] Delete `crates/changala-ring/src/blob.rs`
- [ ] Remove `rust-s3`, `sha2`, `hex` from `changala-ring/Cargo.toml` (now transitive via `atrg-blob`)

#### Step 5: Replace email / OTP

- [ ] Replace `crate::email::send_otp_email()` with `atrg_email::send_otp()`
- [ ] Replace OTP verification logic in `identity.rs` with `atrg_email::verify_otp()`
- [ ] Replace `crate::email::SmtpConfig` with atrg-email's config (loaded via `load_app_config`)
- [ ] Use `atrg_email::validate_domain()` for email domain allowlist checks
- [ ] Verify dev-mode fallback (atrg-email logs OTPs to stdout when SMTP not configured)
- [ ] Delete `crates/changala-ring/src/email.rs`
- [ ] Remove `lettre` from `changala-ring/Cargo.toml` (now transitive via `atrg-email`)

#### Step 6: Replace migration runners

- [ ] Replace `run_changala_migrations()` in Ring `main.rs` with:
  `atrg_db::run_isolated_migrations(&pool, Path::new("./ring_migrations"), "_ring_migrations")`
- [ ] Replace `run_aggregator_migrations()` in Aggregator `main.rs` with:
  `atrg_db::run_isolated_migrations(&pool, Path::new("./aggregator_migrations"), "_aggregator_migrations")`
- [ ] **CRITICAL**: Before first startup, copy existing migration tracking rows:
  ```sql
  INSERT INTO _ring_migrations (version, description, checksum, applied_at)
  SELECT version, description, checksum, installed_on FROM _changala_migrations
  WHERE version NOT IN (SELECT version FROM _ring_migrations);
  ```
- [ ] Delete the custom migration runner functions from both `main.rs` files

#### Step 7: Replace API key authentication

- [ ] Delete `crates/changala-ring/src/api_key_auth.rs`
- [ ] Replace `handlers/apikeys.rs` to use `atrg_auth::api_keys::*`:
  - `create_api_key` → `atrg_auth::api_keys::create_api_key(&pool, did, name, scopes, "chg_")`
  - `list_api_keys` → `atrg_auth::api_keys::list_api_keys(&pool, did)`
  - `revoke_api_key` → `atrg_auth::api_keys::revoke_api_key(&pool, prefix)`
- [ ] Remove `api_key_auth_middleware` from route mounting in Ring `main.rs`
  (atrg `RequireAuth` now handles `chg_*` tokens natively)
- [ ] Remove `mcp_gate_middleware` — replace with `RequireAuth` on MCP routes
- [ ] Verify MCP server still works with API key auth
- [ ] **CRITICAL**: Create RBAC DDL tables before using RBAC functions:
  `atrg_auth::rbac::CREATE_ROLES_TABLE_POSTGRES`
  `atrg_auth::rbac::CREATE_BANS_TABLE_POSTGRES`

#### Step 8: Replace RBAC / auth helpers

- [ ] Replace `handlers/auth.rs` helper functions with `atrg_auth::rbac::*`:
  - `check_not_banned(did)` → `atrg_auth::rbac::is_banned(&pool, did)`
  - `get_role(did)` → `atrg_auth::rbac::has_role(&pool, did, role, resource)`
  - `require_role("admin")` → `atrg_auth::rbac::has_role(&pool, did, "admin", None)`
- [ ] Keep domain-specific helpers that combine RBAC with business logic:
  - `require_enrolled` — checks enrollment table (not pure RBAC)
  - `require_institution_member` — checks membership table
  - `require_class_rep_or_admin` — checks role scoped to a course
  - `get_course_for_session` — pure data lookup
- [ ] Refactor `handlers/auth.rs` to be a thin wrapper over `atrg_auth::rbac`

#### Step 9: Replace config loading + env overrides

- [ ] Define `ChangalaConfig` using `serde::Deserialize` (already exists)
- [ ] Replace manual TOML parsing with `atrg_core::config::load_app_config::<ChangalaConfig>("changala")`
- [ ] Delete `ChangalaConfig::apply_env_overrides()` method (~80 lines)
- [ ] Use atrg's automatic env var overlay (`ATRG_CHANGALA__*` convention) or
  keep a minimal `apply_env_overrides()` for `CHANGALA_*` backward compatibility
- [ ] Replace admin bootstrap logic with `[app] admin_dids` config

#### Step 10: Replace cross-origin auth

- [ ] Add to `atrg.toml`:
  ```toml
  [auth]
  post_login_redirect = "https://changala-app.pages.dev/login"
  ```
- [ ] Remove custom `/auth/complete` endpoint handler from Ring routes
- [ ] Verify frontend receives `?token=&did=&handle=` query params after OAuth

#### Step 11: Replace event router (Aggregator)

- [ ] Replace manual match-dispatch in `events.rs` with `EventRouterBuilder`:
  ```rust
  let router = EventRouterBuilder::new()
      .on_create("app.changala.keyword", handle_keyword)
      .on_create("app.changala.note", handle_note)
      .on_create("app.changala.vote", handle_vote)
      .on_create("app.changala.brain.node", handle_brain_node)
      .on_create("app.changala.brain.link", handle_brain_link)
      .build();
  ```
- [ ] Keep materialisation logic in individual handler functions (domain-specific)
- [ ] Enable cursor persistence: set `cursor = "auto"` in `[jetstream]` config

#### Step 12: Verification

- [ ] `cargo build` — zero errors, zero warnings
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] Run full test suite
- [ ] Verify OAuth login flow works end-to-end
- [ ] Verify API key auth works for MCP
- [ ] Verify OTP email delivery (or dev-mode stdout logging)
- [ ] Verify firehose subscription + materialisation in Aggregator
- [ ] Verify blob upload/download in Ring
- [ ] Smoke test all 62 XRPC endpoints

### 8.6.3 Dependency Changes

**Before (v0.1.3):**

```toml
[workspace.dependencies]
atrg-core = { version = "0.1.3", default-features = false, features = ["postgres"] }
atrg-auth = { version = "0.1.3", default-features = false, features = ["postgres"] }
atrg-db = { version = "0.1.3", default-features = false, features = ["postgres"] }
atrg-xrpc = "0.1.3"
atrg-stream = "0.1.3"
atrg-repo = "0.1.3"
atrg-identity = "0.1.3"

# Hand-written replacements for missing framework features:
rust-s3 = "0.35"        # → replaced by atrg-blob
sha2 = "0.10"           # → replaced by atrg-blob
hex = "0.4"             # → replaced by atrg-blob
lettre = "0.11"         # → replaced by atrg-email
once_cell = "1"         # → replaced by AppState Extensions
```

**After (v0.2.0):**

```toml
[workspace.dependencies]
atrg-core = { version = "0.2.0", default-features = false, features = ["postgres"] }
atrg-auth = { version = "0.2.0", default-features = false, features = ["postgres"] }
atrg-db = { version = "0.2.0", default-features = false, features = ["postgres"] }
atrg-xrpc = "0.2.0"
atrg-stream = "0.2.0"
atrg-repo = "0.2.0"
atrg-identity = "0.2.0"
atrg-blob = "0.2.0"     # NEW — replaces hand-written blob.rs
atrg-email = "0.2.0"    # NEW — replaces hand-written email.rs
atrg-testing = "0.2.0"  # NEW — for handler tests

# REMOVED:
# rust-s3 = "0.35"      — now transitive via atrg-blob
# sha2 = "0.10"         — now transitive via atrg-blob
# hex = "0.4"           — now transitive via atrg-blob
# lettre = "0.11"       — now transitive via atrg-email
# once_cell = "1"       — replaced by AppState Extensions
```

### 8.6.4 Database Migration Considerations

- **Migration tracking tables**: atrg 0.2.0 uses `_atrg_migrations` for framework tables.
  Changala's Ring uses `_changala_migrations` and the Aggregator uses `_changala_aggregator_migrations`.
  After upgrade, switch to `run_isolated_migrations` and use `_ring_migrations` / `_aggregator_migrations`.
  The old tracking tables remain in the DB but are no longer consulted. Copy rows if migrations
  are NOT idempotent (Changala's use `CREATE TABLE IF NOT EXISTS`, so re-application is safe).

- **New framework tables**: atrg 0.2.0's RBAC module requires `atrg_roles` and `atrg_bans` tables.
  These are NOT auto-created — use the DDL constants from `atrg_auth::rbac::CREATE_*_TABLE_POSTGRES`.
  Add a migration file that creates these tables.

- **API key table**: atrg's API key module uses its own table schema. Compare with Changala's existing
  `api_keys` table and either migrate data or start fresh (recommended for MVP).

### 8.6.5 Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| CID format mismatch (atrg-blob vs custom) | 🔴 High | Verify both use `sha256-{hex}` prefix before migration. If different, write a one-time blob re-key migration. |
| API key format change | 🟡 Medium | Regenerate all API keys after migration. Notify MCP users. |
| Migration tracking table confusion | 🟡 Medium | Document which tables are active. Drop old tracking tables after verification. |
| RBAC table schema mismatch | 🟡 Medium | Compare atrg's DDL with Changala's existing schema. Add ALTER TABLE if needed. |
| `apply_env_overrides()` backward compatibility | 🟢 Low | Keep `CHANGALA_*` env vars working via a thin compatibility shim until k8s configs are updated. |
| Firehose cursor format change | 🟢 Low | Start from `"live"` on first run after upgrade. Historical events will be re-processed (materialisation is idempotent). |

---

## Phase 9: Binary Split & Federation Architecture 🚨 CRITICAL

> **This is not post-MVP. This must happen NOW, before more features land.**
> The Ring and Global View (Aggregator) must be separate binaries from the same
> monorepo. Waiting until after MVP makes the split exponentially harder as
> handler code, state, and migrations intertwine further.

### 9.1 Why Split Now

The current codebase ships a **single binary** that serves both Ring endpoints
(`app.changala.ring.*`) and Global View endpoints (`app.changala.globalview.*`)
from the same Axum router, sharing the same Postgres database and state.

This is architecturally wrong for federation:

- **Ring** = per-institution, deployed by each college. Handles writes,
  identity, blob storage, OAuth. Each institution runs their own.
- **Global View (Aggregator)** = one canonical instance at `changala.app`.
  Subscribes to firehoses from ALL Rings, materialises feeds, search indexes,
  brain graph, notifications. Read-only.
- **Frontend** = static SPA on CF Pages. Talks to the **Aggregator only**.
  Institutions deploying Rings don't get a dedicated UI — they connect
  to the shared frontend at `changala-app.pages.dev`.

If we keep them merged, every new Ring feature bleeds into the Aggregator
binary, migrations conflict, and splitting later means rewriting the entire
state layer.

### 9.2 Target Architecture

```
┌────────────────────────────────────────────────────────────────┐
│  changala/ monorepo                                           │
│                                                                │
│  ┌────────────────────┐  ┌────────────────────┐  ┌──────────────┐  │
│  │   changala-ring    │  │ changala-aggregator│  │    web/      │  │
│  │   (binary 1)      │  │   (binary 2)      │  │  (static)   │  │
│  │                    │  │                    │  │              │  │
│  │ • Identity/auth   │  │ • Firehose sub    │  │ • React SPA  │  │
│  │ • Course/session  │  │ • Feeds/search   │  │ • Talks to   │  │
│  │ • Notes/keywords │  │ • Graph index    │  │   Aggregator │  │
│  │ • Brain CRUD     │  │ • Notifications  │  │   only       │  │
│  │ • Blob storage   │  │ • Histograms     │  │              │  │
│  │ • Moderation     │  │ • Multi-Ring     │  │ • Writes go  │  │
│  │ • Archive        │  │   aggregation   │  │   to Ring    │  │
│  │                    │  │                    │  │   via XRPC   │  │
│  │ Deployed by:     │  │ Deployed by:     │  │              │  │
│  │ each institution │  │ changala.app     │  │ CF Pages     │  │
│  └────────┬───────────┘  └────────┬───────────┘  └──────┬───────┘  │
│           │                     │                   │           │
│           └───── ATProto ─────┘                   │           │
│                Firehose                      XRPC calls    │
└────────────────────────────────────────────────────────────────┘
```

### 9.3 Frontend Routing

The frontend (`web/`) is the UI for the **Aggregator**. It is NOT deployed
per-institution. Institutions deploy Rings (headless API servers).

| Frontend URL | Talks to | Purpose |
|---|---|---|
| `changala-app.pages.dev` | Aggregator | Public browsing, brain layer, search, graph |
| `changala-app.pages.dev/login` | Ring (via Aggregator proxy or direct) | AT Protocol OAuth |
| `changala-app.pages.dev/dashboard` | Aggregator + Ring | Dashboard, email verification |
| `changala-app.pages.dev/admin` | Ring (direct) | Institution admin panel |

For writes (create note, enroll, add keyword), the frontend calls the
user's **Ring** directly (identified by institution membership). The
Aggregator never handles writes — it’s read-only.

### 9.4 Monorepo Structure (Target)

```
changala/
├── Cargo.toml              ← workspace root
├── crates/
│   ├── changala-ring/      ← binary 1: Ring server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── handlers/   ← identity, course, session, note, brain, archive, moderation, admin
│   │   │   ├── blob.rs
│   │   │   ├── email.rs
│   │   │   └── state.rs
│   │   └── ring_migrations/
│   ├── changala-aggregator/ ← binary 2: Global View / Aggregator
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── handlers/   ← feed, search, graph, notification, histogram
│   │   │   ├── firehose.rs ← Jetstream subscriber + event processor
│   │   │   └── state.rs
│   │   └── aggregator_migrations/
│   ├── changala-shared/     ← shared library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── types.rs    ← generated types (shared between Ring + Aggregator)
│   │       └── lib.rs
│   └── changala-e2e/        ← integration tests
├── web/                     ← frontend (talks to Aggregator)
├── deploy/
│   ├── ring.Dockerfile
│   ├── aggregator.Dockerfile
│   └── docker-compose.yml
└── lexicons/
```

### 9.5 Split Plan

#### Step 1: Cargo workspace

- [ ] Convert repo to Cargo workspace with `crates/` directory
- [ ] Create `changala-shared` crate for generated types + common helpers
- [ ] Move current `src/generated/` to `changala-shared`

#### Step 2: Extract Ring binary

- [ ] Create `changala-ring` crate
- [ ] Move Ring handlers: `identity`, `course`, `session`, `note`, `brain`,
      `archive`, `moderation`, `admin`
- [ ] Move Ring-specific state: `blob.rs`, `email.rs`
- [ ] Move Ring migrations to `ring_migrations/`
- [ ] Ring has its own `atrg.toml`, Postgres, S3
- [ ] Ring does NOT subscribe to Jetstream — it’s a write server

#### Step 3: Extract Aggregator binary

- [ ] Create `changala-aggregator` crate
- [ ] Move GlobalView handlers: `feed`, `search`, `graph`, `notification`
- [ ] Move Jetstream subscriber (`events.rs`) to Aggregator
- [ ] Move Aggregator migrations to `aggregator_migrations/`
  - Materialised views: keyword histograms, vote counts, brain graph index
  - Notification table
  - Search indexes
- [ ] Aggregator has its own `atrg.toml`, Postgres (no S3 needed)
- [ ] Aggregator subscribes to firehose from one or more Rings

#### Step 4: Frontend re-wiring

- [ ] `VITE_RING_URL` → points to the user’s Ring (derived from institution membership)
- [ ] `VITE_AGGREGATOR_URL` → points to `changala.app` (the canonical Aggregator)
- [ ] XRPC client routes `app.changala.ring.*` → Ring URL
- [ ] XRPC client routes `app.changala.globalview.*` → Aggregator URL
- [ ] This already works — `xrpc.ts` uses `getBaseUrl(nsid)` to split by namespace

#### Step 5: Multi-Ring Aggregation

- [ ] Aggregator config: list of Ring firehose URLs to subscribe to
- [ ] Aggregator stores `institution_did` / `ring_did` on every materialised record
- [ ] Feed/search responses include `institutionDid` for filtering
- [ ] Frontend institution selector / filter

#### Step 6: Separate Docker images + CI

- [ ] `ring.Dockerfile` → `ghcr.io/changala-social/changala-ring`
- [ ] `aggregator.Dockerfile` → `ghcr.io/changala-social/changala-aggregator`
- [ ] Release workflow builds both images
- [ ] Separate Helm charts / k8s manifests for Ring vs Aggregator

### 9.6 Data Flow After Split

```
Institution A (NITC)           Institution B (IITB)
┌──────────────────┐          ┌──────────────────┐
│  Ring (NITC)     │          │  Ring (IITB)     │
│  nitc.changala   │          │  iitb.changala   │
│                  │          │                  │
│ Postgres + S3    │          │ Postgres + S3    │
│ OAuth + identity │          │ OAuth + identity │
│ Courses/sessions │          │ Courses/sessions │
│ Notes/brain CRUD │          │ Notes/brain CRUD │
└────────┬─────────┘          └────────┬─────────┘
         │  ATProto firehose        │
         └────────┬───────────────┘
                  │
         ┌────────┴─────────┐
         │  Aggregator       │
         │  changala.app     │
         │                   │
         │ Feeds + Search    │
         │ Graph index       │
         │ Notifications     │
         │ Cross-institution │
         │ browsing          │
         └────────┬──────────┘
                  │  XRPC
         ┌────────┴─────────┐
         │  Frontend         │
         │  CF Pages          │
         │  (reads → Aggregator)│
         │  (writes → Ring)  │
         └──────────────────┘
```

### 9.7 What’s Already Split-Ready

| Item | Status |
|---|---|
| XRPC namespaces (`ring.*` vs `globalview.*`) | ✅ Clean separation |
| Frontend XRPC routing (`getBaseUrl(nsid)`) | ✅ Already splits by namespace |
| Handler modules (ring handlers vs feed/search/graph) | ✅ Separate files |
| Generated types | ✅ Can be extracted to shared crate |
| Jetstream event handler | ✅ Already in `events.rs`, only Aggregator needs it |
| Migrations | ⚠️ Currently shared — need to split |
| State (`Changala` struct) | ⚠️ Currently shared — Ring needs blob+smtp, Aggregator doesn’t |
| `main.rs` | ❌ Single binary — must split |

### 9.8 Why the Frontend Only Needs the Aggregator

The frontend is a **public browsing + social interaction** surface. Its primary
data source is the Aggregator (feeds, search, graph, notifications). Writes go
to the Ring, but the Ring URL is derived from the user’s institution membership
— the frontend already knows which Ring to talk to via `VITE_RING_URL`.

Institutions deploying a Ring get:
- A headless XRPC API server
- Content storage (Postgres + S3)
- OAuth + identity management
- Firehose publishing for the Aggregator to consume

They do NOT get a dedicated UI. Their users access the shared frontend at
`changala-app.pages.dev` (or `changala.app` post-launch).

---

## Phase 10: API Keys & MCP Server 💡 PLANNED (simplified by atrg 0.2.0)

> **API key infrastructure is now framework-provided by atrg 0.2.0.**
> `atrg-auth` provides `create_api_key`, `list_api_keys`, `revoke_api_key`,
> and transparent `RequireAuth` integration for API key tokens.
> This phase reduces to MCP tool definitions and admin UX only.

> First-class MCP (Model Context Protocol) support on the Ring for AI-powered
> institution management. Eliminates manual frontend labour for bootstrapping,
> bulk operations, and ongoing maintenance.

### 10.1 Why MCP on the Ring

Setting up Changala for an institution involves creating dozens of courses,
hundreds of sessions, enrolling students, assigning class reps, configuring
semesters. Doing this through the admin panel is brutal. An MCP server lets
an AI assistant (Claude, GPT, etc.) do it conversationally:

> "Create all CS department courses for Fall 2026 based on this syllabus PDF"
> "Enroll all students from this spreadsheet into CS301"
> "Open today's sessions and close yesterday's"
> "Show me all students who haven't submitted keywords this week"

MCP lives on the **Ring** because:
- All write operations are Ring endpoints (`app.changala.ring.*`)
- Institution-scoped — each Ring serves one institution
- Admin operations require Ring-level auth, not Aggregator
- The Aggregator is read-only — MCP needs writes

Post-MVP, students get limited MCP access for their own brain nodes.

### 10.2 API Key System (Prerequisite)

MCP servers authenticate via API keys, not OAuth. The Ring needs an API key
system that maps keys to DIDs + roles.

**Schema:**

```sql
CREATE TABLE api_keys (
    id BIGSERIAL PRIMARY KEY,
    key_hash TEXT NOT NULL UNIQUE,     -- SHA-256 of the API key
    key_prefix TEXT NOT NULL,          -- first 8 chars for identification
    did TEXT NOT NULL,                 -- owner DID
    name TEXT NOT NULL,                -- human-readable label
    scopes TEXT NOT NULL DEFAULT '[]', -- JSON array of allowed scopes
    expires_at TEXT,                   -- optional expiry
    created_at TEXT NOT NULL,
    last_used_at TEXT
);
```

**Scopes:**

| Scope | What it allows |
|---|---|
| `admin:*` | All admin operations (courses, roles, moderation, archives) |
| `admin:courses` | Create/list courses, assign class reps |
| `admin:sessions` | Session lifecycle (create, open, close, cancel) |
| `admin:moderation` | Ban/unban users |
| `admin:roles` | Promote/demote users |
| `write:notes` | Create/version notes |
| `write:brain` | Create/version brain nodes and links |
| `read:*` | Read any public data |

**Endpoints:**

| Method | Endpoint | Auth | Description |
|---|---|---|---|
| `POST` | `ring.createApiKey` | Admin | Generate a new API key |
| `GET` | `ring.listApiKeys` | Admin | List all keys (prefix + name, not full key) |
| `POST` | `ring.revokeApiKey` | Admin | Revoke a key by prefix |

**Usage:** `Authorization: Bearer chg_xxxxxxxxxxxxxxxxxxxx`

API keys are prefixed with `chg_` for easy identification.
The `RequireAuth` extractor is extended to accept API keys alongside
OAuth session tokens — it checks the `api_keys` table if the token
starts with `chg_`.

**Env vars:**

| Env Var | Description |
|---|---|
| `CHANGALA_BOOTSTRAP_API_KEY` | Pre-provisioned admin API key for first-time setup (no frontend needed) |

### 10.3 MCP Server

The MCP server runs as a **sidecar mode** of `changala-ring` (same binary,
different entrypoint) or as a separate lightweight process that calls the
Ring's XRPC endpoints using an API key.

**Architecture option A: Built into the Ring (recommended)**

```
changala-ring --mcp      → starts MCP server on stdio/SSE
changala-ring             → starts normal XRPC server
```

The MCP server uses the same handler code but exposes it via MCP protocol
instead of HTTP. No network hop, same process, same DB.

**Architecture option B: Standalone MCP proxy**

A thin MCP server that translates MCP tool calls into XRPC HTTP calls
to the Ring. Simpler to implement but adds a network hop.

### 10.4 MCP Tools (Admin)

| Tool | Ring endpoint | Description |
|---|---|---|
| `create_course` | `ring.createCourse` | Create a course with title, code, dept, semester |
| `list_courses` | `ring.listCourses` | List courses with filters |
| `create_session` | `ring.createSession` | Schedule a session |
| `open_session` | `ring.openSession` | Transition session to live |
| `close_session` | `ring.closeSession` | End session, open keyword window |
| `bulk_create_courses` | Multiple `ring.createCourse` | Create N courses from structured input |
| `bulk_enroll` | Multiple `ring.enrollStudent` | Enroll list of DIDs into a course |
| `assign_class_rep` | `ring.assignClassRep` | Assign class rep to a course |
| `promote_role` | `ring.promoteRole` | Change a user's role |
| `ban_user` | `ring.banDid` | Ban a DID |
| `list_bans` | `ring.listBans` | View active bans |
| `initiate_archive` | `ring.initiateArchive` | Start archival for a semester |
| `verify_email` | `ring.verifyEmail` | Manually verify a user's email |
| `get_audit_log` | `ring.getAuditLog` | View recent admin actions |
| `provision_admin` | `ring.provisionAdmin` | Bootstrap admin via shared secret |

### 10.5 MCP Tools (Student — Post-MVP)

| Tool | Ring endpoint | Description |
|---|---|---|
| `create_brain_node` | `ring.createNode` | Create a brain node |
| `create_link` | `ring.createLink` | Link two brain nodes |
| `create_note` | `ring.createNote` | Submit a note for a session |
| `add_keyword` | `ring.addKeyword` | Submit a keyword |
| `my_brain_nodes` | `globalview.getBrainFeed` | List own brain nodes |
| `search` | `globalview.search*` | Search across content |

### 10.6 MCP Resources

| Resource URI | Description |
|---|---|
| `changala://courses` | Course catalog |
| `changala://courses/{uri}` | Single course detail |
| `changala://sessions/{uri}` | Session detail + keyword histogram |
| `changala://brain/{uri}` | Brain node content + backlinks |
| `changala://audit-log` | Recent admin actions |
| `changala://bans` | Active ban list |

### 10.7 Implementation Checklist

**API Keys:**

- [ ] Migration: `api_keys` table
- [ ] `POST ring.createApiKey` / `GET ring.listApiKeys` / `POST ring.revokeApiKey`
- [ ] Extend `RequireAuth` extractor to accept `chg_` prefixed API keys
- [ ] `CHANGALA_BOOTSTRAP_API_KEY` env var for first-time setup
- [ ] API key scopes enforcement

**MCP Server:**

- [ ] Add `mcp-server` crate to workspace (or feature flag on `changala-ring`)
- [ ] Rust MCP SDK integration ([`mcp-rust-sdk`](https://github.com/modelcontextprotocol/rust-sdk) or hand-rolled)
- [ ] Admin tools (15 tools mapping to Ring endpoints)
- [ ] MCP resources for read access
- [ ] `changala-ring --mcp` entrypoint (stdio transport)
- [ ] SSE transport option for remote MCP access
- [ ] Student-scoped MCP tools (post-MVP)

**Frontend (Admin Panel):**

- [ ] API key management tab: create, list, revoke
- [ ] Copy-to-clipboard for new API keys
- [ ] MCP connection instructions display

### 10.8 Usage Example

```
# Bootstrap: generate an admin API key
curl -X POST https://ring.changala.app/xrpc/app.changala.ring.createApiKey \
  -H "Authorization: Bearer <session>" \
  -d '{"name": "MCP Admin Key", "scopes": ["admin:*"]}'
# → {"key": "chg_abc123...", "prefix": "chg_abc1", ...}

# Configure Claude Desktop / Cursor:
{
  "mcpServers": {
    "changala": {
      "command": "changala-ring",
      "args": ["--mcp"],
      "env": {
        "CHANGALA_API_KEY": "chg_abc123...",
        "CHANGALA_RING_URL": "https://ring.changala.app"
      }
    }
  }
}

# Then in conversation:
> "Create CS301 Algorithms, CS302 Data Structures, and CS303 OS for Fall 2026"
> "Enroll all 120 CS students into CS301"
> "Assign did:plc:xyz as class rep for CS301"
```

---

## Phase 11: Enhanced Features (Post-MVP)

- [ ] Faculty verification and endorsement labels
- [ ] Push notifications (replacing MVP polling)
- [ ] Scanned document → LaTeX conversion
- [ ] Client-side wikilink auto-parsing (`[[wikilink]]` → link records)
- [ ] Feed ranking algorithms (social graph weighted)

---

## Out of Scope (Permanently)

| Item | Rationale |
|---|---|
| Per-institution frontend | Institutions deploy Rings (headless API). All users share the canonical frontend at `changala-app.pages.dev`. |
| Private brain nodes | Openness is the point. If you don't want it public, don't put it on Changala. |
| Wikipedia integration | Changala archive is the canonical source. |
| PDS implementation | atrg builds ON TOP of the AT Protocol network. Users bring their own PDS. |
