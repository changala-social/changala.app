# Changala — RBAC Reference

> Role-Based Access Control specification for the Changala platform.
> Covers role hierarchy, permission matrix, auth flow, ban system, bootstrapping, and proposed extensions.

---

## 1. Role Hierarchy

Changala uses a simple numeric hierarchy. Lower level = more privilege.

| Level | Role | Scope |
|-------|------------|-------|
| 1 | `admin` | Instance-wide |
| 2 | `classRep` | Per-course |
| 3 | `student` | Per-institution |
| — | `faculty` | *(post-MVP, not yet implemented)* |

A user with level N can perform any action requiring level ≥ N. Concretely:

- **Admin (1)** can do everything a Class Rep and Student can do, plus admin-only operations.
- **Class Rep (2)** can do everything a Student can do, plus session lifecycle and edit acceptance — but only for the course(s) they represent.
- **Student (3)** can read all public content, enroll in courses, write notes, keywords, votes, brain nodes, and links.

### Scope Note

`classRep` is a **per-course** role. A user may be a Class Rep for CS301 but a regular Student in EE201. The `require_class_rep_or_admin` check always takes a course ID and verifies the user holds the `classRep` role for *that specific course* (or is an `admin`, which bypasses course scoping).

---

## 2. How Roles Are Assigned

| Role | Assignment Mechanism | Who Can Assign |
|------|----------------------|----------------|
| `student` | Automatic on successful `verifyEmail` | System (self-service) |
| `classRep` | `POST assignClassRep` | `admin` |
| `admin` | Direct DB insert | Instance operator (manual) |
| `faculty` | *(post-MVP)* | *(separate verification flow)* |

### Flow

```/dev/null/flow.txt#L1-7
1. User authenticates via AT Protocol OAuth → gets an atrg_session
2. User calls POST verifyEmail (step 1) with their institution email
   → System generates OTP, logs it (email sending not yet implemented)
3. User calls POST verifyEmail (step 2) with the OTP
   → System creates a row in the `memberships` table with role = "student"
4. Admin can later promote a student to classRep via POST assignClassRep
5. Admin accounts are currently created by direct SQL insert (see §8)
```

---

## 3. Role Descriptions

### Student

The default role. Granted to any user who verifies their institution email address. Students can:

- Enroll in courses
- Submit keywords for sessions
- Write and version personal notes (LaTeX, markdown, etc.)
- Propose edits to collective notes
- Vote on notes and brain nodes
- Create, version, link, and delete brain nodes
- Apply and retract labels
- Export archives
- Receive and manage notifications

### Class Rep

A per-course elevated role assigned by an admin. In addition to all student capabilities, a Class Rep can:

- Create, open, close, cancel, and reschedule sessions for their assigned course
- Accept or reject collective note edit proposals for their assigned course

Class Rep permissions are **scoped to the course** they were assigned to. A user can be Class Rep for multiple courses.

### Admin

Instance-wide superuser. In addition to all student and Class Rep capabilities (across all courses), an admin can:

- Create courses
- Assign Class Reps to courses
- Ban and unban DIDs (with optional TTL)
- List all bans
- Initiate and seal semester archives
- Trigger uploads to the Internet Archive

### Faculty *(post-MVP)*

Not yet implemented. Intended capabilities:

- Apply endorsement labels with faculty authority
- Verified via a separate institutional verification flow
- No session lifecycle or admin powers unless also assigned those roles

---

## 4. Permission Matrix

### 4.1 Public Endpoints (No Auth Required)

These endpoints are accessible to anyone, including unauthenticated users.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Server info |
| `/api/health` | GET | Health check |
| `/client-metadata.json` | GET | OAuth client metadata |
| `/.well-known/oauth-protected-resource` | GET | OAuth protected resource metadata |
| `verifyEmail` | POST | Email verification (step 1: send OTP, step 2: confirm OTP) |
| `getMemberships` | GET | List memberships for a DID |
| `getRole` | GET | Get role for a DID at an institution |
| `getCourse` | GET | Get course details |
| `listCourses` | GET | List all courses |
| `getSession` | GET | Get session details |
| `listSessions` | GET | List sessions for a course |
| `getEnrollments` | GET | List enrollments for a course |
| `getNoteContent` | GET | Get note content blob from Ring |
| `getNoteHistory` | GET | Get version history of a note |
| `getCollectiveNote` | GET | Get the collective note for a session |
| `listEditProposals` | GET | List edit proposals for a collective note |
| `getNodeContent` | GET | Get brain node content blob from Ring |
| `isBanned` | GET | Check if a DID is banned |
| `getArchive` | GET | Get archive details |

#### Global View Public Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `getNotes` | GET | List notes (with mode filter) |
| `getCourseFeed` | GET | Feed of activity for a course |
| `getSocialFeed` | GET | Academic activity from followed DIDs |
| `getBrainFeed` | GET | Brain node activity from followed DIDs |
| `getTrendingKeywords` | GET | Trending keywords across sessions |
| `getTrendingBrainTags` | GET | Trending brain node tags |
| `getFollowedEnrollments` | GET | Courses that followed users are enrolled in |
| `getGlobalArchiveFeed` | GET | Recently sealed archives across Rings |
| `getKeywordHistogram` | GET | Keyword frequency for a session |
| `searchNotes` | GET | Full-text search over notes |
| `searchCourses` | GET | Search courses |
| `searchArchive` | GET | Search archived content |
| `searchBrainNodes` | GET | Search brain nodes |
| `getNodeGraph` | GET | Get brain node graph neighbourhood |
| `getBacklinks` | GET | Get backlinks to a brain node |
| `getNeighbours` | GET | Get nodes within N hops |

### 4.2 Authenticated Endpoints (Any Verified User)

These require `RequireAuth` (valid AT Protocol session) + `check_not_banned`.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `enrollStudent` | POST | Enroll in a course |
| `addKeyword` | POST | Submit a keyword for a session |
| `createNote` | POST | Create a new note |
| `versionNote` | POST | Create a new version of a note |
| `proposeEdit` | POST | Propose an edit to the collective note |
| `registerVote` | POST | Upvote a note or brain node |
| `applyLabel` | POST | Apply a quality label |
| `retractLabel` | POST | Retract a previously applied label |
| `createNode` | POST | Create a brain node |
| `versionNode` | POST | Version a brain node |
| `createLink` | POST | Create a link between brain nodes |
| `deleteLink` | POST | Delete a brain link |
| `exportArchive` | POST | Export an archive bundle |
| `getNotifications` | GET | Get notifications for the current user |
| `markNotificationRead` | POST | Mark a notification as read |
| `markAllRead` | POST | Mark all notifications as read |

### 4.3 Class Rep or Admin Endpoints

These require `RequireAuth` + `require_class_rep_or_admin(course_id)`. The user must be an admin **or** a Class Rep for the specific course in question.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `createSession` | POST | Create a new session for a course |
| `openSession` | POST | Open a session (set status to Live) |
| `closeSession` | POST | Close a session (set status to Ended) |
| `cancelSession` | POST | Cancel a session |
| `rescheduleSession` | POST | Reschedule a session |
| `acceptEdit` | POST | Accept a collective note edit proposal |
| `rejectEdit` | POST | Reject a collective note edit proposal |

### 4.4 Admin-Only Endpoints

These require `RequireAuth` + `require_role("admin")`.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `createCourse` | POST | Create a new course |
| `assignClassRep` | POST | Assign a user as Class Rep for a course |
| `banDid` | POST | Ban a DID (permanent or with TTL) |
| `liftBan` | POST | Remove a ban |
| `listBans` | GET | List all active bans |
| `initiateArchive` | POST | Begin the archival process for a course |
| `sealArchive` | POST | Seal an archive (make immutable) |
| `uploadToInternetArchive` | POST | Upload sealed archive to archive.org |

---

## 5. Auth Flow

```/dev/null/auth-flow.txt#L1-20
┌──────────┐    OAuth 2.0     ┌──────────┐
│  Client  │ ───────────────► │   PDS    │
│          │ ◄─────────────── │(Bluesky) │
│          │   access_token   └──────────┘
│          │
│          │   access_token    ┌──────────────────────────┐
│          │ ─────────────────►│     Changala Ring         │
└──────────┘                   │                          │
                               │  1. RequireAuth extractor│
                               │     → validates token    │
                               │     → extracts DID       │
                               │     → creates atrg_session│
                               │                          │
                               │  2. check_not_banned(did)│
                               │     → queries bans table │
                               │                          │
                               │  3. Role check (if needed)│
                               │     → require_role("admin")│
                               │     or                   │
                               │     → require_class_rep_or_admin(course_id)│
                               └──────────────────────────┘
```

### Step by Step

1. **OAuth**: The client authenticates with the user's PDS using AT Protocol OAuth 2.0. The PDS issues an access token.

2. **RequireAuth**: On every authenticated request, the `RequireAuth` extractor validates the access token, extracts the user's DID, and establishes an `atrg_session`. If the token is invalid or expired, the request is rejected with `401 Unauthorized`.

3. **Ban Check**: For all write operations, `check_not_banned(did)` queries the `bans` table. If the DID has an active ban (no expiry, or expiry in the future), the request is rejected with `403 Forbidden`.

4. **Role Check** (when required):
   - `require_role("admin")` — looks up the user's membership and asserts `role = "admin"`. Returns `403` if not.
   - `require_class_rep_or_admin(course_id)` — looks up the user's membership and checks that either `role = "admin"` **or** (`role = "classRep"` **and** the user is the assigned Class Rep for the given `course_id`). Returns `403` if neither condition holds.

---

## 6. Ban System

Bans are stored in the `bans` table and checked on every authenticated write operation via `check_not_banned`.

### Ban Properties

| Field | Type | Description |
|-------|------|-------------|
| `did` | string | The banned DID |
| `reason` | string | Human-readable reason for the ban |
| `expires_at` | datetime? | `NULL` = permanent, otherwise ban lifts at this time |
| `created_at` | datetime | When the ban was created |
| `created_by` | string | DID of the admin who issued the ban |

### Behaviour

- **Permanent ban**: `expires_at` is `NULL`. The DID cannot perform any write operation until an admin calls `liftBan`.
- **Temporary ban (TTL)**: `expires_at` is set to a future timestamp. The ban automatically expires — `check_not_banned` compares against the current time.
- **Read access**: Banned users can still read all public endpoints. Bans only block writes.
- **Ban check scope**: The ban check runs after `RequireAuth` succeeds but before any business logic. A banned user's existing content (notes, brain nodes, etc.) remains visible.

### Admin Operations

| Endpoint | Effect |
|----------|--------|
| `POST banDid` | Creates a ban entry (permanent or with TTL) |
| `POST liftBan` | Removes the ban entry for a DID |
| `GET listBans` | Lists all active bans (admin only) |
| `GET isBanned` | Check if a specific DID is banned (public) |

---

## 7. Bootstrapping

### The First Admin Problem

There is no API endpoint to create the first admin. This is intentional — the admin role grants instance-wide control, and the initial admin must be provisioned out-of-band.

### Current Method: Direct DB Insert

Connect to the Ring's database and insert a membership row with `role = 'admin'`:

```/dev/null/bootstrap.sql#L1-9
-- Bootstrap the first admin for an institution
-- Replace the DID and institution values with your own

INSERT INTO memberships (did, institution, role, verified_email, created_at)
VALUES (
  'did:plc:your-admin-did-here',
  'nitc.ac.in',
  'admin',
  'admin@nitc.ac.in',
  NOW()
);
```

To find your DID, resolve your AT Protocol handle:

```/dev/null/resolve-did.sh#L1-2
# Resolve a handle to a DID
curl https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle=yourhandle.bsky.social
```

### Proposed: CHANGALA_ADMIN_DIDS Environment Variable

For easier deployment, the Ring should support an environment variable that auto-provisions admin memberships on startup:

```/dev/null/env-example.txt#L1-5
# Comma-separated list of DIDs to auto-provision as admins
# The Ring checks this on startup and ensures each DID has an admin membership
# If a membership already exists, it is upgraded to admin
# If no membership exists, one is created with institution = instance default
CHANGALA_ADMIN_DIDS=did:plc:abc123,did:plc:def456
```

This would run during Ring initialization, before the server starts accepting requests.

---

## 8. Proposed: Admin Provisioning Endpoint

**Status**: Not yet implemented. Proposed for near-term development.

### Motivation

Requiring direct DB access to create admins is workable for a single operator but doesn't scale to deployments where the instance operator may not have (or want) SQL access. A provisioning endpoint secured by a shared secret solves this.

### Design

```/dev/null/provision-admin.txt#L1-16
POST /xrpc/app.changala.ring.provisionAdmin

Headers:
  Authorization: Bearer <CHANGALA_ADMIN_SECRET>

Body:
{
  "did": "did:plc:target-user-did",
  "institution": "nitc.ac.in"
}

Response (200):
{
  "did": "did:plc:target-user-did",
  "role": "admin",
  "institution": "nitc.ac.in"
}
```

### Security

- The `CHANGALA_ADMIN_SECRET` environment variable holds a high-entropy shared secret (minimum 32 characters).
- This endpoint does **not** use AT Protocol OAuth. It uses a simple `Bearer` token check against the env var.
- If `CHANGALA_ADMIN_SECRET` is not set, the endpoint returns `501 Not Implemented`.
- The endpoint should be rate-limited (e.g. 5 requests per minute) and log every invocation.
- This endpoint creates or upgrades a membership to `admin`. It does not require the target DID to have verified their email first.

### Implementation Checklist

- [ ] Add `CHANGALA_ADMIN_SECRET` to the Ring's config/env loader
- [ ] Add the `/xrpc/app.changala.ring.provisionAdmin` route
- [ ] Implement Bearer token validation (constant-time comparison)
- [ ] Upsert into `memberships` with `role = 'admin'`
- [ ] Add rate limiting
- [ ] Add audit logging
- [ ] Add lexicon JSON for `app.changala.ring.provisionAdmin`

---

## 9. Proposed: Email Verification

**Status**: OTPs are currently logged to stdout. No emails are actually sent.

### Current Behaviour

```/dev/null/current-otp.txt#L1-5
1. Client calls POST verifyEmail { email: "student@nitc.ac.in" }
2. Ring generates a 6-digit OTP
3. Ring logs the OTP to stdout (for development)
4. Client calls POST verifyEmail { email: "student@nitc.ac.in", otp: "123456" }
5. Ring verifies the OTP and creates a student membership
```

### What Needs to Happen

To move from development to production, the Ring needs to actually deliver OTPs via email. Two approaches:

#### Option A: SMTP Integration

```/dev/null/smtp-env.txt#L1-5
CHANGALA_SMTP_HOST=smtp.gmail.com
CHANGALA_SMTP_PORT=587
CHANGALA_SMTP_USER=noreply@changala.app
CHANGALA_SMTP_PASS=app-specific-password
CHANGALA_SMTP_FROM=Changala <noreply@changala.app>
```

Use the `lettre` crate for Rust SMTP support. Send a simple plaintext email with the OTP.

#### Option B: Third-Party Email API

Use a transactional email service (Resend, Postmark, SendGrid, etc.) via their HTTP API. Simpler to set up, better deliverability, but adds an external dependency.

### Implementation Checklist

- [ ] Choose email delivery method (SMTP vs. API)
- [ ] Add email config to env/config loader
- [ ] Implement email sending in the `verifyEmail` handler
- [ ] Add OTP expiry (e.g. 10 minutes)
- [ ] Add rate limiting on OTP requests (e.g. 3 per email per hour)
- [ ] Add email template (plain text is fine for MVP)

---

## 10. Proposed: Super Admin

**Status**: Post-MVP. Relevant only when federation is live.

### Motivation

When multiple institutions run their own Rings and a canonical Global View aggregates them, there needs to be a role that operates above individual instance admins — for managing federation policy, cross-instance moderation, and Global View configuration.

### Proposed Hierarchy

| Level | Role | Scope |
|-------|------|-------|
| 0 | `superAdmin` | Cross-instance / Global View |
| 1 | `admin` | Single institution Ring |
| 2 | `classRep` | Per-course within an institution |
| 3 | `student` | Per-institution |

### Super Admin Capabilities (Proposed)

- Manage the Ring allowlist on the Global View (which Rings to subscribe to)
- Cross-instance moderation (ban a DID across all federated Rings)
- Global View configuration (search index settings, feed ranking parameters)
- Override instance-level admin decisions in exceptional cases
- Manage federation policy (which data crosses instance boundaries)

### Not in Scope for Super Admin

- Course or session management (that's instance-level admin work)
- Student-facing features (Super Admin is infrastructure, not pedagogy)

---

## Appendix A: Quick Reference Table

| Endpoint | Auth | Role Required | Scope |
|----------|------|---------------|-------|
| `GET /` | None | — | — |
| `GET /api/health` | None | — | — |
| `GET /client-metadata.json` | None | — | — |
| `GET /.well-known/oauth-protected-resource` | None | — | — |
| `POST verifyEmail` | None | — | — |
| `GET getMemberships` | None | — | — |
| `GET getRole` | None | — | — |
| `GET getCourse` | None | — | — |
| `GET listCourses` | None | — | — |
| `GET getSession` | None | — | — |
| `GET listSessions` | None | — | — |
| `GET getEnrollments` | None | — | — |
| `GET getNoteContent` | None | — | — |
| `GET getNoteHistory` | None | — | — |
| `GET getCollectiveNote` | None | — | — |
| `GET listEditProposals` | None | — | — |
| `GET getNodeContent` | None | — | — |
| `GET isBanned` | None | — | — |
| `GET getArchive` | None | — | — |
| All Global View GETs | None | — | — |
| `POST enrollStudent` | RequireAuth | `student` (any) | + ban check |
| `POST addKeyword` | RequireAuth | `student` (any) | + ban check |
| `POST createNote` | RequireAuth | `student` (any) | + ban check |
| `POST versionNote` | RequireAuth | `student` (any) | + ban check |
| `POST proposeEdit` | RequireAuth | `student` (any) | + ban check |
| `POST registerVote` | RequireAuth | `student` (any) | + ban check |
| `POST applyLabel` | RequireAuth | `student` (any) | + ban check |
| `POST retractLabel` | RequireAuth | `student` (any) | + ban check |
| `POST createNode` | RequireAuth | `student` (any) | + ban check |
| `POST versionNode` | RequireAuth | `student` (any) | + ban check |
| `POST createLink` | RequireAuth | `student` (any) | + ban check |
| `POST deleteLink` | RequireAuth | `student` (any) | + ban check |
| `POST exportArchive` | RequireAuth | `student` (any) | + ban check |
| `GET getNotifications` | RequireAuth | `student` (any) | — |
| `POST markNotificationRead` | RequireAuth | `student` (any) | + ban check |
| `POST markAllRead` | RequireAuth | `student` (any) | + ban check |
| `POST createSession` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST openSession` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST closeSession` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST cancelSession` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST rescheduleSession` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST acceptEdit` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST rejectEdit` | RequireAuth | `classRep` / `admin` | course-scoped |
| `POST createCourse` | RequireAuth | `admin` | instance-wide |
| `POST assignClassRep` | RequireAuth | `admin` | instance-wide |
| `POST banDid` | RequireAuth | `admin` | instance-wide |
| `POST liftBan` | RequireAuth | `admin` | instance-wide |
| `GET listBans` | RequireAuth | `admin` | instance-wide |
| `POST initiateArchive` | RequireAuth | `admin` | instance-wide |
| `POST sealArchive` | RequireAuth | `admin` | instance-wide |
| `POST uploadToInternetArchive` | RequireAuth | `admin` | instance-wide |

---

*Document created — living reference, updated as auth requirements evolve.*
