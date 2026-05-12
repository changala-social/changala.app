# Changala — MVP Specification
> A federated, open-protocol social learning platform and student knowledge graph, built on the AT Protocol.
> *"Preserve knowledge. Make it social. Give it to anyone who seeks."*

---

## 0. North Star

Changala is not an LMS replacement. It is two things that belong together:

1. **A student-owned, collectively maintained knowledge trail** of every class session — open to the world, federated across institutions, designed to outlive the semester it was born in.
2. **A public second brain for every student** — a living, interconnected Zettelkasten of everything they know: academic notes, personal insights, extracurricular learning, ideas that don't fit a syllabus.

The AT Protocol provides the backbone: portable identity, a social graph, and a firehose of events. Changala adds the academic and knowledge graph layer on top.

---

## 1. MVP Scope

**Target institution:** NIT Calicut (NITC) — single Ring deployment, single Global View.
**Goal:** Prove both the academic loop and the brain loop work with real students.
**Deliverable:** A well-documented **API gateway** (gRPC + REST) built on [`at-rust-go`](https://github.com/tellmeY18/at-rust-go), shipping two binaries from one monorepo — the **Ring** (content + identity server) and the **Global View** (aggregator). UI/UX is out of scope for MVP.

---

## 2. System Architecture

Changala ships three logical components from **one monorepo**:

```
┌──────────────────────────────────────────────────────────────┐
│                         changala/                            │
│                                                              │
│   ┌──────────────┐   ┌──────────────┐   ┌────────────────┐  │
│   │     Ring     │   │ Global View  │   │  API Gateway   │  │
│   │              │   │              │   │                │  │
│   │ Heavy blob   │   │ Firehose     │   │ gRPC + REST    │  │
│   │ storage:     │   │ subscriber + │   │ Business logic │  │
│   │ LaTeX, PDFs, │   │ read-layer   │   │ Auth, writes   │  │
│   │ markdown,    │   │ aggregator   │   │                │  │
│   │ archives,    │   │ search index │   │                │  │
│   │ brain nodes  │   │ graph index  │   │                │  │
│   └──────┬───────┘   └──────┬───────┘   └───────┬────────┘  │
└──────────┼─────────────────-┼───────────────────┼───────────┘
           │                  │                   │
           └──────── ATProto Firehose / PDS ───────┘
```

### 2.1 PDS (User-owned, external)
The user's Personal Data Server — Bluesky-hosted or self-hosted. Holds **only minimal lexicon records**: membership credentials, keywords, vote signals, note metadata pointers, brain node metadata. The PDS never holds large blobs.

### 2.2 Ring (Changala's Knot equivalent)
A lightweight, self-hostable content server. Holds **heavy content** that doesn't belong on a PDS:
- LaTeX source files and compiled PDFs (academic notes)
- Markdown / plaintext blobs (brain nodes)
- Version diffs (collective note history, brain node edits)
- Archived semester bundles
- Question paper uploads

Each institution deploying Changala runs one Ring. ATProto records on the PDS contain a `ring_ref` pointing to the actual content blob on the Ring.

### 2.3 Global View (Changala's AppView equivalent)
A read-layer aggregator subscribing to the **ATProto firehose** (via Jetstream). Materialises:
- Keyword histograms per session
- Vote counts per note
- Academic and brain feeds
- Full-text search (academic notes + brain nodes)
- **Brain graph index** — backlinks, outbound links, graph traversal
- Cross-institution browsing

One canonical Global View will be deployed at `changala.app` post-MVP. In MVP it runs alongside the NITC Ring.

---

## 3. Framework Strategy — at-rust-go (atrg) 0.2.0

**Lexicon JSON files are the source of truth. Rust types are auto-generated from them.**

`at-rust-go` (atrg) is the batteries-included backend framework Changala is built on. As of **v0.2.0** (released 2026-05-12), the framework provides far more than just codegen — it is the full application runtime.

### 3.1 What atrg Provides Natively

| Capability | atrg Crate | Changala Usage |
|---|---|---|
| **Lexicon codegen** | `atrg-codegen` | 74 lexicon JSON files → 156 Rust structs, 62 handler stubs |
| **OAuth + Session management** | `atrg-auth` | AT Protocol OAuth login/logout, session cookies, JWT verification |
| **API Key authentication** | `atrg-auth` | `chg_*` prefixed keys for MCP and programmatic access |
| **RBAC (roles + bans)** | `atrg-auth` | `has_role`, `grant_role`, `revoke_role`, `ban_did`, `lift_ban` with TTL |
| **Blob storage** | `atrg-blob` | S3-compatible content-addressed storage (`BlobStore` trait) |
| **Email / OTP** | `atrg-email` | SMTP delivery + two-step OTP verification flow |
| **Jetstream consumer** | `atrg-stream` | Firehose subscription with backpressure, reconnection, ZSTD |
| **Event router** | `atrg-stream` | `EventRouterBuilder` — typed dispatch by collection + operation |
| **Cursor persistence** | `atrg-stream` | Auto-save/resume last processed event across restarts |
| **XRPC endpoints** | `atrg-xrpc` | AT Protocol error envelope, route helpers |
| **DID/handle resolution** | `atrg-identity` | TTL-backed cache for identity lookups |
| **AppState extensions** | `atrg-core` | Type-safe app state via `with_extension::<T>()` — no `once_cell` |
| **App-specific config** | `atrg-core` | `load_app_config::<T>("changala")` from `atrg.toml` |
| **Migration isolation** | `atrg-db` | Separate tracking tables per binary (`_ring_migrations`, `_aggregator_migrations`) |
| **Admin bootstrap** | `atrg-core` | `[app] admin_dids` auto-provisions admin roles on startup |
| **Cross-origin auth** | `atrg-auth` | `[auth] post_login_redirect` for SPA token handoff |
| **Rate limiting** | `atrg-core` | Per-IP token-bucket middleware from `[rate_limit]` config |
| **Feed generator** | `atrg-feed` | Feed skeleton XRPC routes (if needed post-MVP) |
| **Label service** | `atrg-label` | Label creation, signing, storage, query |
| **Testing utilities** | `atrg-testing` | Mock clients, fake Jetstream, in-memory test state |
| **Multi-binary scaffold** | `atrg-cli` | `atrg new --template multi-binary` (write server + aggregator) |

### 3.2 What Changala Hand-Writes

Only **business logic** — the domain-specific parts that no framework can provide:

- Handler implementations (course lifecycle, session management, note workflows, brain graph, archive pipeline)
- SQL queries and migrations (16 tables, domain-specific schemas)
- Permission logic beyond basic RBAC (e.g. "class rep for *this specific course*")
- Firehose materialisation logic (what to index, how to aggregate)
- Notification dispatch rules

### 3.3 Build Pipeline

```
lexicons/app/changala/*.json    (74 files — records + XRPC procedures/queries)
        │
        ▼  (atrg generate)
crates/changala-shared/src/generated/
        ├── types.rs             ← 156 ATProto record types + validators
        └── routes.rs            ← 62 Axum handler stubs with typed I/O
        │
crates/changala-ring/src/        ← Ring handler implementations (write server)
crates/changala-aggregator/src/  ← Aggregator handler implementations (read layer)
```

Lexicons live in `lexicons/` at the repo root. Proto files are preserved as reference documentation in `docs/proto/`. Generated code goes in `crates/changala-shared/src/generated/` and is gitignored.

---

## 4. The Two Layers

Changala has two distinct but connected layers. A **mode switch** in the client (and corresponding feed filter in the Global View API) toggles between them.

```
┌─────────────────────────────────────────────────────────┐
│                     CHANGALA                            │
│                                                         │
│  ┌──────────────────────┐  ┌────────────────────────┐  │
│  │   ACADEMIC LAYER     │  │     BRAIN LAYER        │  │
│  │                      │  │                        │  │
│  │  Courses             │  │  Brain nodes           │  │
│  │  Sessions            │  │  (any topic, any       │  │
│  │  Class notes (LaTeX) │  │   format)              │  │
│  │  Collective notes    │  │  Backlinks             │  │
│  │  Keyword histograms  │  │  Tags                  │  │
│  │  Archives            │  │  Graph view            │  │
│  │                      │  │  Linked to academic    │  │
│  │  ← course-bound →    │  │  or free-floating      │  │
│  └──────────────────────┘  └────────────────────────┘  │
│                                                         │
│  [Academic Focus Mode]  ←switch→  [Full Brain Mode]     │
└─────────────────────────────────────────────────────────┘
```

**Academic Focus Mode** — Feed and search surface only course sessions, class notes, collective notes, keyword histograms, archives. Clean, distraction-free academic view.

**Full Brain Mode** — Everything above, plus brain nodes from people you follow: personal insights, extracurricular learning, project notes, book notes, ideas — anything a student wants to preserve and make public.

The two layers are **linked**. A brain node can reference a course session or a class note. A class note can link to a brain node. The graph index in the Global View knows about both.

---

## 5. Identity & Roles

### 5.1 AT Protocol Identity
Every user brings their own AT Protocol handle. Changala never owns your identity. All records live on your DID.

### 5.2 Institution Verification
- OTP/magic link sent to institution email (`user@nitc.ac.in`)
- On success: writes `app.changala.membership` record to user's PDS
- AT Protocol account and institution email are fully decoupled
- Multiple institution memberships supported — one email verification per institution
- DNS-based handle verification: not required in MVP

### 5.3 Roles

| Role | How Obtained | Capabilities |
|---|---|---|
| **Student** | Institution email verified | Enroll in courses, write notes, keywords, votes, brain nodes |
| **Class Rep** | Assigned by admin or course consensus | Session lifecycle management, collective note acceptance |
| **Platform Admin** | Instance operator | Course creation, Class Rep assignment, moderation, federation config |
| **Faculty** *(post-MVP)* | Separate verification | Apply endorsement labels |

---

## 6. Visibility Model

| Tier | Visible To | Examples |
|---|---|---|
| **World Public** | Anyone, unauthenticated | Archives, brain nodes (default), course index, Global View |
| **Institution Public** | Verified members of the instance | Active session feeds, timetables, live histograms |
| **Course-Enrolled** | Students enrolled in a specific course | Session notes, collective note drafts, edit proposals |

**Brain nodes are world-public by default.** The entire premise of the brain layer is radical openness — if you don't want it public, don't put it on Changala. There is no private tier.

---

## 7. Data Model

### 7.1 What Lives Where

| Record | PDS | Ring | Global View |
|---|---|---|---|
| `app.changala.membership` | ✓ tiny | — | indexed |
| `app.changala.keyword` | ✓ tiny | — | histogram materialised |
| `app.changala.note` (metadata) | ✓ small | blob | indexed |
| `app.changala.vote` | ✓ tiny | — | count materialised |
| `app.changala.collectivenote.proposal` | ✓ small | diff blob | indexed |
| `app.changala.label` | ✓ tiny | — | signal materialised |
| `app.changala.brain.node` (metadata) | ✓ small | blob | indexed + graph indexed |
| `app.changala.brain.link` | ✓ tiny | — | graph indexed |
| `app.changala.archive` | Ring DID repo | bundle blob | indexed |
| `app.changala.course` | Ring DID repo | — | indexed |
| `app.changala.session` | Ring DID repo | — | indexed |
| Note / brain content blobs | — | ✓ | — |
| Keyword histogram | — | — | ✓ materialised |
| Brain graph (backlinks, neighbours) | — | — | ✓ materialised |

### 7.2 Lexicon Namespace — `app.changala.*`

Informal and unregistered in MVP. Validated instance-side. Formal registration post-MVP.

```
── Academic layer ──────────────────────────────────────────────
app.changala.membership              — institution affiliation credential
app.changala.course                  — course/subject record (Ring DID repo)
app.changala.session                 — single class period (Ring DID repo)
app.changala.keyword                 — student keyword for a session
app.changala.note                    — note metadata + ring_ref
app.changala.collectivenote.proposal — proposed edit to collective note
app.changala.vote                    — upvote on a note or brain node
app.changala.label                   — knowledge quality label
app.changala.archive                 — sealed end-of-semester record

── Brain layer ─────────────────────────────────────────────────
app.changala.brain.node              — a single knowledge node
app.changala.brain.link              — a directed link between two nodes
```

### 7.3 Brain Node Record

```
node_id       : string (rkey)
title         : string           — short title of the node
format        : "markdown"
              | "plaintext"
              | "latex"          — latex allowed if it's academic content
              | "html"
ring_ref      : RingRef          — pointer to content blob on Ring
tags          : list[string]     — free-form tags (e.g. "physics", "book-notes", "idea")
academic_ref  : at-uri?          — optional link to a course or session this relates to
version       : int
parent_node   : at-uri?          — previous version (versioned like notes)
summary       : string?          — short description (shown in graph tooltips)
created_at    : datetime
updated_at    : datetime
```

### 7.4 Brain Link Record

```
from_uri   : at-uri     — AT URI of the source brain node
to_uri     : at-uri     — AT URI of the target (brain node, note, session, course, or
                           external URL stored as a string)
label      : string?    — optional edge label (e.g. "see also", "contradicts", "expands")
created_at : datetime
```

Backlinks are derived by the Global View's graph index — if node B has a link to node A, the Global View surfaces node B as a backlink on node A's view. No separate backlink record needed.

---

## 8. The Brain Layer — Behaviour

### 8.1 What a Brain Node Is

A brain node is anything a student wants to capture and make public:
- A concept they learned outside class
- Notes on a book they read
- A project they're working on
- A thought about a topic from an unexpected angle
- An extracurricular skill (music theory, competitive programming tricks, cooking techniques)
- A connection they noticed between two ideas

It is **not** tied to a course or session. It can optionally reference one via `academic_ref` if it grew out of a class, but it stands on its own.

### 8.2 Linking

Links between nodes are first-class records (`app.changala.brain.link`). When writing a brain node in markdown, the client may parse `[[wikilink]]` syntax and auto-create link records for each. The Global View maintains a bidirectional graph from these records:

- **Outbound links** — nodes this node explicitly links to
- **Backlinks** — nodes that link to this node (derived, not stored)
- **Neighbours** — nodes reachable within 2 hops (for graph view)

### 8.3 Academic Links

A brain node with `academic_ref` set to a session URI appears in both the brain graph and the academic feed for that session. This is the bridge between the two layers — a student writing a brain node that expands on something covered in CS301 contributes to both their personal graph and the collective course knowledge.

### 8.4 Format

Brain nodes support **markdown** (default), plaintext, LaTeX, or HTML. No restriction to LaTeX like academic notes — most non-academic content is naturally markdown. Format is declared in the record and the Ring stores the raw source; clients render it.

### 8.5 Tags and Map of Content

Tags on brain nodes are how students build their own Map of Content. The Global View surfaces:
- All a user's nodes grouped by tag
- Trending tags across the institution
- Tag-based search

### 8.6 Mode Switch

The mode switch is a **query parameter on feed and search endpoints**, not a separate API. The Global View's `GetFeed` and `Search` endpoints accept a `mode` parameter:

- `mode=academic` — returns only course/session-bound content
- `mode=brain` — returns only brain layer content
- `mode=all` (default) — returns everything

The client decides what to show. The API never enforces a view.

---

## 9. Firehose & Event Model

The Global View subscribes to the **ATProto firehose** (via Jetstream).

### 9.1 Event → Action Map

| Firehose Event | Global View Action |
|---|---|
| `app.changala.keyword` created | Increment keyword histogram for session |
| `app.changala.note` created/versioned | Index in search, update course feed |
| `app.changala.vote` created | Increment vote count for target |
| `app.changala.collectivenote.proposal` created | Queue for Class Rep, notify |
| `app.changala.label` created | Apply label signal to target |
| `app.changala.session` status changed | Trigger notifications to enrolled students |
| `app.changala.archive` sealed | Move to immutable archive tier |
| `app.changala.brain.node` created/versioned | Index in search + brain graph index |
| `app.changala.brain.link` created | Update bidirectional graph index |

### 9.2 Resync
Global View is fully reconstructible by replaying `subscribeRepos` from all known PDSes. No data is lost — PDS is source of truth for all small records.

---

## 10. Social Graph

Read-only from ATProto. No mutations.

| Feature | How the Social Graph is Used |
|---|---|
| **Onboarding** | "3 people you follow are enrolled in CS301 at NITC" |
| **Academic feed ranking** | Notes from people you follow surface higher |
| **Brain feed** | Brain nodes from people you follow appear in Full Brain Mode |
| **Discovery** | "People you follow are writing about these topics" |
| **Graph overlay** | In the brain graph view, edges from followed users are highlighted |

---

## 11. Notification Model

Stored on the Ring, delivered via API polling in MVP (no push).

| Event | Recipients | Priority |
|---|---|---|
| Session opened | All enrolled students | Normal |
| Session cancelled | All enrolled students | **High** |
| Session rescheduled | All enrolled students | **High** |
| Keyword window closing (10 min) | Enrolled students who haven't added a keyword | Normal |
| Vote received on your note | Note author | Low |
| Vote received on your brain node | Node author | Low |
| Collective note edit proposed | Class Rep | Normal |
| Collective note edit accepted | Proposal author | Low |
| Semester archive initiated | All enrolled students | Normal |
| Label applied to your content | Content author | Normal |
| Someone linked to your brain node | Node author | Low |

---

## 12. Session Lifecycle

```
Scheduled → [Open] → Live → [Close] → Ended
                                         │
                              keyword_window = true (60 min)
                                         │
                              keyword_window = false
                                         │
                           [notes + brain nodes editable all semester]
                                         │
                              [semester end → SealArchive]
                                         │
                                     Archived (immutable)

Scheduled → [Cancel] → Cancelled
Scheduled → [Reschedule] → Rescheduled
```

---

## 13. Archival Pipeline

Manual. Triggered by collective consensus (admin + Class Rep + community).

### 13.1 What Gets Archived

| Item | Destination | Format |
|---|---|---|
| Collective notes (best-voted) | Ring archive + `archive.org` | LaTeX / PDF |
| Question papers | `archive.org` | PDF |
| Keyword histogram | Ring archive | JSON |
| Academic brain nodes with `academic_ref` to this course | Ring archive | Markdown / LaTeX |
| Additional materials (community decision) | `archive.org` | Original format |

> Pure brain nodes (no academic_ref) are **never archived** by the platform — they live on the student's own PDS and Ring reference indefinitely, independent of semester lifecycle.

### 13.2 Archive Properties
- Immutable once sealed
- `archive.org` upload: explicit admin trigger only, never automatic
- No Wikipedia integration — Changala archive is the canonical source
- Wikitext export available as a utility format

---

## 14. Content Discovery & Search

Global View maintains a search index over all world-public records — both academic and brain.

### 14.1 Search

| Mode | Searches across |
|---|---|
| `mode=academic` | Note content, collective notes, keyword histograms, course/session metadata |
| `mode=brain` | Brain node content, tags, titles |
| `mode=all` | Everything above |

### 14.2 Discovery Feeds

- **Course feed** — all session activity for a course
- **Social feed (academic)** — academic activity from followed DIDs
- **Social feed (brain)** — brain node activity from followed DIDs
- **Trending keywords** — institution-wide keyword signal across active sessions
- **Trending brain tags** — most-used tags across brain nodes this week
- **Global archive feed** — recently sealed courses from any Ring on the network
- **Graph feed** — brain nodes that link to content you've written (your node's neighbourhood)

---

## 15. Federation Model

```
nitc.changala.app   (Ring) ─┐
iitb.changala.app   (Ring) ─┼──► changala.app (Global View)
kerala.changala.app (Ring) ─┘
```

- Self-hostable — each institution runs a Ring
- Instance admins can restrict cross-instance interaction (override, not default)
- `changala.app` is the canonical Global View post-MVP
- Brain nodes are visible across all instances by default — the knowledge graph is global
- MVP: one Ring (NITC) + local Global View

---

## 16. API Surface Summary

Two binaries. Service definitions in `.proto` files. Record types auto-generated from lexicon JSON by `at-rust-go`.

### Ring Services

| Service | Key RPCs |
|---|---|
| `IdentityService` | `VerifyEmail`, `GetMemberships`, `GetRole` |
| `CourseService` | `CreateCourse`, `EnrollStudent`, `AssignClassRep`, `ListCourses` |
| `SessionService` | `CreateSession`, `OpenSession`, `CloseSession`, `CancelSession`, `RescheduleSession` |
| `NoteService` | `CreateNote`, `VersionNote`, `GetNoteContent`, `GetNoteHistory`, `ProposeEdit`, `AcceptEdit`, `GetCollectiveNote` |
| `BrainService` | `CreateNode`, `VersionNode`, `GetNodeContent`, `CreateLink`, `DeleteLink` |
| `VoteService` | `RegisterVote` |
| `LabelService` | `ApplyLabel`, `RetractLabel` |
| `ModerationService` | `BanDID`, `LiftBan`, `ListBans` |
| `ArchiveService` | `InitiateArchive`, `SealArchive`, `ExportArchive`, `UploadToInternetArchive` |

### Global View Services

| Service | Key RPCs |
|---|---|
| `FeedService` | `GetCourseFeed`, `GetSocialFeed`, `GetBrainFeed`, `GetTrendingKeywords`, `GetTrendingBrainTags`, `GetFollowedEnrollments`, `GetGlobalArchiveFeed` |
| `SearchService` | `SearchNotes`, `SearchBrainNodes`, `SearchCourses`, `SearchArchive` |
| `HistogramService` | `GetKeywordHistogram` |
| `GraphService` | `GetNodeGraph`, `GetBacklinks`, `GetNeighbours` |
| `NotificationService` | `GetNotifications`, `MarkNotificationRead`, `MarkAllRead` |

---

## 17. What MVP Does NOT Include

| Item | Status |
|---|---|
| UI / UX | Out of scope — API first |
| AI features | Deferred — opt-in, student collective consent required |
| Faculty verification | Post-MVP |
| Federation with other Rings | Post-MVP (NITC only in MVP) |
| Scanned doc → LaTeX | Future |
| DNS-based institution handle proof | Post-MVP |
| Push notifications | Post-MVP — polling in MVP |
| Brain node wikilink auto-parse | Client-side concern; Ring accepts link records directly |
| Wikipedia integration | Dropped permanently |
| Private brain nodes | Not planned — openness is the point |

---

## 18. Resolved Decisions

| Decision | Resolution | Rationale |
|---|---|---|
| **Lexicon generation** | Auto-generated from JSON by `at-rust-go` — no hand-written Rust types | Framework handles this; proto files cover only gRPC service interfaces |
| **Proto files** | Service definitions only — not data types | Data types come from lexicon codegen |
| **PDS scope** | Minimal metadata + small records only | Heavy blobs belong on Ring |
| **Ring** | Lightweight self-hostable blob server | Changala's Knot equivalent |
| **Global View** | Same monorepo, separate binary; firehose subscriber + read layer | Mirrors Tangled's AppView; `changala.app` is canonical post-MVP |
| **Firehose** | Jetstream subscription | Reactive materialisation — histograms, graph, feeds |
| **Social graph** | Read-only from ATProto | Free infrastructure; Changala never mutates it |
| **Institution proof** | Email OTP only | DNS is too high a bar for student-bootstrapped adoption |
| **Brain node visibility** | World-public by default, no private tier | Openness is the core value |
| **Brain format** | Markdown / plaintext / LaTeX / HTML | Academic = LaTeX preferred; brain = markdown default |
| **Mode switch** | Query param on feed + search endpoints — `mode=academic|brain|all` | Client decides the view; API never enforces it |
| **Brain archival** | Only nodes with `academic_ref` get archived at semester end | Pure brain nodes live independently of semester lifecycle |
| **Keyword histogram** | Materialised on Global View via firehose | Fast reads; resync-able from PDS |
| **Collective note acceptance** | Class Rep only in MVP | No gameable auto-accept thresholds in MVP |
| **Keyword attribution** | Per-user on PDS; anonymous in histogram API | Privacy in the read layer |
| **Moderation** | Instance-side deny list (DID + optional TTL) | Pragmatic for MVP; ATProto labels are long-term path |
| **Labels** | `app.changala.label` — ATProto-native quality signal | Portable, verifiable |
| **Offline draft sync** | Backend accepts timestamp-preserving writes within semester | Client queues; Ring enforces semester boundary |
| **Framework version** | atrg 0.2.0 — batteries-included | Native API keys, RBAC, blob storage, email/OTP, event router, cursor persistence, admin bootstrap, cross-origin auth, AppState extensions, migration isolation |
| **State management** | `AtrgApp::with_extension::<T>()` — no `once_cell` globals | Framework-managed typed state, accessible via `state.extension::<T>()` in handlers |
| **API key auth** | Native `atrg-auth` API keys (`chg_*` prefix) | No more synthetic session upserts; `RequireAuth` handles API keys transparently |
| **RBAC** | `atrg-auth` RBAC (`has_role`, `grant_role`, `ban_did`) | Replace hand-rolled SQL role checks with framework-provided functions |
| **Blob storage** | `atrg-blob` S3BlobStore | Replace custom 97-line blob module with framework `BlobStore` trait |
| **Email delivery** | `atrg-email` SMTP + OTP | Replace custom 78-line email module with framework `send_otp`/`verify_otp` |
| **Migration isolation** | `run_isolated_migrations` with per-binary tracking tables | Replace custom 40-line migration runners in both binaries |
| **Cross-origin auth** | `[auth] post_login_redirect` config | Replace custom `/auth/complete` endpoint with framework config |
| **Admin bootstrap** | `[app] admin_dids` config + env var | Replace manual startup provisioning SQL |
| **App config** | `load_app_config::<ChangalaConfig>("changala")` | Replace manual TOML parsing + 80-line `apply_env_overrides()` |
| **Event routing** | `EventRouterBuilder` with `.on_create()` per collection | Replace manual match-dispatch in aggregator `events.rs` |

---

## 19. Next Steps

- [x] ~~Add `app.changala.brain.node` and `app.changala.brain.link` lexicon JSON files~~
- [x] ~~Set up `at-rust-go` project scaffold and point it at the lexicons directory~~
- [x] ~~Implement Ring: `IdentityService` + `CourseService` as first milestone~~
- [x] ~~Implement Global View: Jetstream subscriber + keyword histogram materialisation as first milestone~~
- [ ] **Migrate to atrg 0.2.0** — replace ~800 lines of hand-written framework workarounds with native atrg features (see ROADMAP.md Phase 8.6)
- [ ] Complete identity, email verification & RBAC using atrg-native modules (see ROADMAP.md Phase 8.5)
- [ ] Define archival export bundle format (LaTeX + markdown, `archive.org` metadata schema)
- [ ] Binary split into Cargo workspace (see ROADMAP.md Phase 9)

---

*Document generated from brainstorm session — May 2026. Living document, subject to revision.*
*Renamed from Honey Heave → **Changala** (chain, in Malayalam).*
*Brain layer added: Zettelkasten-style public second brain for every student.*
*atrg 0.2.0 migration plan added — May 2026.*
