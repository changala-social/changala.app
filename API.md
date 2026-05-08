# Changala — Backend API Specification

> Complete XRPC endpoint reference for frontend integration.
> Base URL: `https://your-ring.example.com` (default port 3000)

---

## Table of Contents

- [Protocol Overview](#protocol-overview)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [Pagination](#pagination)
- [Common Types](#common-types)
- [Ring Endpoints](#ring-endpoints)
  - [Identity Service](#identity-service)
  - [Course Service](#course-service)
  - [Session Service](#session-service)
  - [Note Service](#note-service)
  - [Collective Note Service](#collective-note-service)
  - [Vote & Label Service](#vote--label-service)
  - [Brain Service](#brain-service)
  - [Moderation Service](#moderation-service)
  - [Archive Service](#archive-service)
- [Global View Endpoints](#global-view-endpoints)
  - [Feed Service](#feed-service)
  - [Search Service](#search-service)
  - [Graph Service](#graph-service)
  - [Notification Service](#notification-service)
- [Firehose Events](#firehose-events)
- [Env Var Overrides](#env-var-overrides)

---

## Protocol Overview

Changala uses **XRPC** (Cross-Reference Procedure Call), the AT Protocol's RPC framework.

- **Procedures** (writes): `POST /xrpc/<nsid>` with JSON body
- **Queries** (reads): `GET /xrpc/<nsid>?param=value`
- **Content-Type**: Always `application/json`
- **Namespace**: `app.changala.ring.*` for Ring (write/content server), `app.changala.globalview.*` for Global View (read/aggregator)

All endpoints return JSON. All timestamps are ISO 8601 / RFC 3339 strings (e.g. `"2025-02-01T10:00:00Z"`).

### Non-XRPC Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/` | Server info: `{ name, description, status }` |
| `GET` | `/api/health` | Health check: `{ healthy: true }` |

---

## Authentication

Changala uses AT Protocol OAuth (PKCE + DPoP). Authenticated endpoints require a valid bearer token in the `Authorization` header:

```
Authorization: Bearer <access_token>
```

The token is obtained through the standard AT Protocol OAuth flow via the `/auth/callback` endpoint.

### Auth Levels

| Level | Description | Endpoints |
|---|---|---|
| **Public** | No auth required | All `GET` queries, `verifyEmail` |
| **Authenticated** | Any verified user | `enrollStudent`, `addKeyword`, `createNote`, `versionNote`, `proposeEdit`, `registerVote`, `applyLabel`, `retractLabel`, `createNode`, `versionNode`, `createLink`, `deleteLink`, `exportArchive`, all notifications |
| **Class Rep / Admin** | Class rep for the course, or admin | `createSession`, `openSession`, `closeSession`, `cancelSession`, `rescheduleSession`, `acceptEdit`, `rejectEdit` |
| **Admin only** | Platform administrator | `createCourse`, `assignClassRep`, `banDid`, `liftBan`, `listBans`, `initiateArchive`, `sealArchive`, `uploadToInternetArchive` |

Unauthenticated requests to protected endpoints return HTTP `401`.
Insufficient role returns HTTP `403` with a descriptive message.
Banned users receive HTTP `403` with `"This account has been suspended"`.

---

## Error Handling

All errors return JSON with this shape:

```json
{
  "error": "InvalidRequest",
  "message": "Human-readable description of what went wrong"
}
```

### Error Codes

| HTTP Status | Error Name | When |
|---|---|---|
| `400` | `InvalidRequest` | Validation failure, duplicate record, wrong state transition |
| `401` | `AuthenticationRequired` | Missing or invalid bearer token |
| `403` | `Forbidden` | Insufficient role or account suspended |
| `404` | `NotFound` | Resource doesn't exist |
| `500` | `InternalServerError` | Database or blob store failure |

---

## Pagination

Paginated endpoints use cursor-based pagination:

- **Request**: `?limit=50&cursor=<opaque_string>`
- **Response**: `{ ..., cursor: "<next_cursor>" | null }`

- `limit`: Number of items per page (default varies by endpoint, max 100, min 1)
- `cursor`: Omit for the first page. Use the `cursor` from the previous response for the next page.
- When `cursor` is `null` in the response, there are no more pages.

---

## Common Types

### RingRef

Pointer to a content blob stored on the Ring's S3-compatible blob store.

```json
{
  "ringDid": "did:web:ring.changala.local",
  "cid": "sha256-a1b2c3d4e5f6..."
}
```

| Field | Type | Description |
|---|---|---|
| `ringDid` | `string` (DID) | DID of the Ring instance hosting the blob |
| `cid` | `string` | Content-addressed identifier (SHA-256 hex) |

### AT URI

AT Protocol URI format: `at://<did>/<collection>/<rkey>`

Example: `at://did:plc:abc123/app.changala.brain.node/3jui7kd2zc22a`

### Session Status

`"scheduled"` | `"live"` | `"ended"` | `"cancelled"` | `"rescheduled"`

### Note Format

`"latex"` | `"plaintext"` | `"markdown"` | `"html"`

### Role

`"student"` | `"classRep"` | `"admin"`

### Visibility

`"world"` | `"institution"` | `"course"`

---

## Ring Endpoints

### Identity Service

#### `POST /xrpc/app.changala.ring.verifyEmail`

Two-step OTP email verification flow. No auth required.

**Step 1 — Request OTP:**

```json
// Request
{ "did": "did:plc:abc123", "email": "student@nitc.ac.in" }

// Response (200)
{ "status": "otpSent", "membershipUri": null }
```

**Step 2 — Verify OTP:**

```json
// Request
{ "did": "did:plc:abc123", "email": "student@nitc.ac.in", "otp": "482910" }

// Response (200)
{ "status": "verified", "membershipUri": "at://did:plc:abc123/app.changala.membership/3jui7kd2zc22a" }
```

| Field | Type | Required | Description |
|---|---|---|---|
| `did` | `string` | ✅ | User's AT Protocol DID |
| `email` | `string` | ✅ | Institution email address |
| `otp` | `string` | ❌ | 6-digit OTP (omit for step 1) |

**Errors:** `InvalidRequest` — invalid/expired OTP

---

#### `GET /xrpc/app.changala.ring.getMemberships`

List all institution memberships for a DID. No auth required.

| Param | Type | Required |
|---|---|---|
| `did` | `string` | ✅ |

```json
// Response (200)
{
  "memberships": [
    {
      "institutionDid": "did:web:nitc-ac-in",
      "institutionDomain": "nitc.ac.in",
      "role": "student",
      "verifiedAt": "2025-01-15T10:00:00Z",
      "membershipUri": ""
    }
  ]
}
```

---

#### `GET /xrpc/app.changala.ring.getRole`

Get a user's role. No auth required.

| Param | Type | Required |
|---|---|---|
| `did` | `string` | ✅ |

```json
// Response (200)
{ "role": "student" }
```

**Errors:** `NotFound` — no verified membership

---

### Course Service

#### `POST /xrpc/app.changala.ring.createCourse`

Create a new course. **Admin only.**

```json
// Request
{
  "title": "Introduction to Algorithms",
  "code": "CS301",
  "department": "Computer Science",
  "semester": "Fall 2025",
  "visibility": "institution",
  "description": "Fundamental algorithms and data structures"
}

// Response (200)
{
  "uri": "at://did:web:ring.changala.local/app.changala.course/3jui7kd2zc22a",
  "title": "Introduction to Algorithms",
  "code": "CS301",
  "department": "Computer Science",
  "semester": "Fall 2025",
  "visibility": "institution",
  "description": "Fundamental algorithms and data structures",
  "createdBy": "did:plc:admin123",
  "classRepDid": null,
  "enrolledCount": 0,
  "createdAt": "2025-01-01T00:00:00Z"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `title` | `string` | ✅ | Course title |
| `code` | `string` | ✅ | Course code (e.g. "CS301") |
| `department` | `string` | ✅ | Department name |
| `semester` | `string` | ✅ | Semester identifier (e.g. "Fall 2025") |
| `visibility` | `string` | ✅ | `"world"`, `"institution"`, or `"course"` |
| `description` | `string` | ❌ | Course description |

**Errors:** `InvalidRequest` — duplicate code+semester · `Forbidden` — not admin

---

#### `GET /xrpc/app.changala.ring.getCourse`

Get a single course by URI. No auth required.

| Param | Type | Required |
|---|---|---|
| `uri` | `string` (AT URI) | ✅ |

Response: Same shape as `createCourse` output (with `enrolledCount`).

**Errors:** `NotFound`

---

#### `GET /xrpc/app.changala.ring.listCourses`

List courses with optional filters. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `semester` | `string` | ❌ | — |
| `department` | `string` | ❌ | — |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
// Response (200)
{
  "courses": [ /* array of course objects */ ],
  "cursor": "2025-01-01T00:00:00Z"
}
```

---

#### `POST /xrpc/app.changala.ring.enrollStudent`

Enroll in a course. **Auth required.**

```json
// Request
{ "courseUri": "at://did:web:ring/app.changala.course/abc", "targetDid": "did:plc:student1" }

// Response (200)
{ "courseUri": "at://...", "did": "did:plc:student1", "enrolledAt": "2025-01-15T10:00:00Z" }
```

| Field | Type | Required | Description |
|---|---|---|---|
| `courseUri` | `string` | ✅ | Course AT URI |
| `targetDid` | `string` | ❌ | DID to enroll (defaults to caller's DID) |

**Errors:** `NotFound` — course doesn't exist · `InvalidRequest` — already enrolled

---

#### `GET /xrpc/app.changala.ring.getEnrollments`

List enrolled DIDs for a course. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `courseUri` | `string` | ✅ | — |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
// Response (200)
{ "courseUri": "at://...", "dids": ["did:plc:a", "did:plc:b"], "total": 42, "cursor": null }
```

---

#### `POST /xrpc/app.changala.ring.assignClassRep`

Assign a class representative for a course. **Admin only.**

```json
// Request
{ "courseUri": "at://...", "classRepDid": "did:plc:student1" }

// Response (200) — full course view with updated classRepDid
```

---

### Session Service

All session responses share this shape:

```json
{
  "uri": "at://did:web:ring/app.changala.session/abc",
  "courseUri": "at://...",
  "scheduledAt": "2025-02-01T10:00:00Z",
  "durationMins": 60,
  "status": "scheduled",
  "createdBy": "did:plc:admin",
  "topic": "Sorting Algorithms",
  "openedAt": null,
  "closedAt": null,
  "keywordWindowExpiresAt": null,
  "keywordWindowOpen": false,
  "rescheduledTo": null,
  "createdAt": "2025-01-20T00:00:00Z"
}
```

#### `POST /xrpc/app.changala.ring.createSession`

Create a new session. **Class rep or admin.**

| Field | Type | Required | Description |
|---|---|---|---|
| `courseUri` | `string` | ✅ | Course AT URI |
| `scheduledAt` | `string` | ✅ | ISO 8601 datetime |
| `durationMins` | `integer` | ✅ | Duration in minutes |
| `topic` | `string` | ❌ | Session topic |

---

#### `GET /xrpc/app.changala.ring.getSession`

| Param | Type | Required |
|---|---|---|
| `uri` | `string` | ✅ |

`keywordWindowOpen` is computed dynamically based on `keywordWindowExpiresAt` vs current time.

---

#### `GET /xrpc/app.changala.ring.listSessions`

| Param | Type | Required | Default |
|---|---|---|---|
| `courseUri` | `string` | ✅ | — |
| `statuses` | `string[]` | ❌ | all statuses |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

---

#### `POST /xrpc/app.changala.ring.openSession`

Transition: `scheduled` → `live`. **Class rep or admin.**

```json
{ "sessionUri": "at://..." }
```

---

#### `POST /xrpc/app.changala.ring.closeSession`

Transition: `live` → `ended`. Opens a 60-minute keyword window. **Class rep or admin.**

```json
// Request
{ "sessionUri": "at://..." }

// Response
{ "session": { /* session view */ }, "keywordWindowExpiresAt": "2025-02-01T12:00:00Z" }
```

---

#### `POST /xrpc/app.changala.ring.cancelSession`

Transition: `scheduled` or `live` → `cancelled`. **Class rep or admin.**

```json
{ "sessionUri": "at://...", "reason": "Lecturer unavailable" }
```

---

#### `POST /xrpc/app.changala.ring.rescheduleSession`

Transition: `scheduled` → `rescheduled`. **Class rep or admin.**

```json
{ "sessionUri": "at://...", "rescheduledTo": "2025-02-03T10:00:00Z" }
```

---

### Note Service

#### `POST /xrpc/app.changala.ring.addKeyword`

Submit a keyword for a session. **Auth required.** Only works when the keyword window is open (session is `live`, or within 60 min after `closeSession`).

```json
// Request
{ "sessionUri": "at://...", "text": "quicksort" }

// Response (200)
{ "keywordUri": "at://did:plc:user/app.changala.keyword/abc", "windowExpiresAt": "2025-02-01T12:00:00Z" }
```

**Errors:** `InvalidRequest` — window closed, or duplicate keyword by same user · `NotFound` — session not found

---

#### `POST /xrpc/app.changala.ring.createNote`

Create a note for a session. **Auth required.** Content is stored in the Ring's S3 blob store.

```json
// Request
{
  "sessionUri": "at://...",
  "content": "\\section{Quicksort}...",
  "format": "latex",
  "summary": "Notes on sorting algorithm complexity",
  "createdAt": "2025-02-01T11:30:00Z"
}

// Response (200)
{
  "ringRef": { "ringDid": "did:web:ring.changala.local", "cid": "sha256-abc123" },
  "noteTemplate": {
    "uri": "at://did:plc:user/app.changala.note/abc",
    "sessionUri": "at://...",
    "format": "latex",
    "ringRef": { "ringDid": "...", "cid": "..." },
    "version": 1,
    "summary": "Notes on sorting algorithm complexity",
    "createdAt": "2025-02-01T11:30:00Z"
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `sessionUri` | `string` | ✅ | Session AT URI |
| `content` | `string` | ✅ | Note content (raw text) |
| `format` | `string` | ✅ | `"latex"`, `"plaintext"`, `"markdown"`, `"html"` |
| `summary` | `string` | ❌ | Short description |
| `createdAt` | `string` | ❌ | ISO 8601 (defaults to server time) |

---

#### `POST /xrpc/app.changala.ring.versionNote`

Create a new version of an existing note. **Auth required.**

| Field | Type | Required |
|---|---|---|
| `parentNoteUri` | `string` | ✅ |
| `content` | `string` | ✅ |
| `format` | `string` | ✅ |
| `summary` | `string` | ❌ |
| `createdAt` | `string` | ❌ |

Response: Same as `createNote`, with incremented `version` and `parentNote` field.

---

#### `GET /xrpc/app.changala.ring.getNoteContent`

Fetch raw note content from the blob store. No auth required.

| Param | Type | Required |
|---|---|---|
| `cid` | `string` | ✅ |

```json
// Response (200)
{ "format": "latex", "content": "\\section{Quicksort}..." }
```

---

#### `GET /xrpc/app.changala.ring.getNoteHistory`

Get all versions of a note. No auth required.

| Param | Type | Required |
|---|---|---|
| `noteUri` | `string` | ✅ |

```json
// Response (200)
{
  "sessionUri": "at://...",
  "authorDid": "did:plc:user",
  "versions": [
    { "uri": "at://...", "ringRef": {...}, "version": 1, "summary": "...", "createdAt": "..." },
    { "uri": "at://...", "ringRef": {...}, "version": 2, "parentNoteUri": "at://...", "summary": "...", "createdAt": "..." }
  ]
}
```

---

### Collective Note Service

#### `POST /xrpc/app.changala.ring.proposeEdit`

Propose an edit to the collective note. **Auth required.**

```json
// Request
{ "sessionUri": "at://...", "diff": "--- a/note\n+++ b/note\n...", "summary": "Added complexity analysis" }

// Response (200)
{ "proposalUri": "at://...", "diffRingRef": { "ringDid": "...", "cid": "..." } }
```

---

#### `POST /xrpc/app.changala.ring.acceptEdit`

Accept a pending edit proposal. **Class rep or admin.**

```json
// Request
{ "proposalUri": "at://..." }

// Response (200)
{ "collectiveNoteUri": "at://...", "newCollectiveRingRef": { "ringDid": "...", "cid": "..." } }
```

---

#### `POST /xrpc/app.changala.ring.rejectEdit`

Reject a pending edit proposal. **Class rep or admin.**

```json
{ "proposalUri": "at://..." }
// Response: { "proposalUri": "at://..." }
```

---

#### `GET /xrpc/app.changala.ring.getCollectiveNote`

Get the current collective note for a session. No auth required.

| Param | Type | Required |
|---|---|---|
| `sessionUri` | `string` | ✅ |

```json
// Response (200)
{
  "sessionUri": "at://...",
  "ringRef": { "ringDid": "...", "cid": "..." },
  "contributorDids": ["did:plc:a", "did:plc:b"],
  "version": 3,
  "updatedAt": "2025-02-05T00:00:00Z"
}
```

---

#### `GET /xrpc/app.changala.ring.listEditProposals`

List edit proposals for a session. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `sessionUri` | `string` | ✅ | — |
| `statusFilter` | `string` | ❌ | all (`"pending"`, `"accepted"`, `"rejected"`) |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
// Response (200)
{
  "proposals": [
    {
      "proposalUri": "at://...",
      "sessionUri": "at://...",
      "proposerDid": "did:plc:user",
      "diffRingRef": { "ringDid": "...", "cid": "..." },
      "status": "pending",
      "summary": "Added complexity analysis",
      "resolvedAt": null,
      "resolvedBy": null,
      "createdAt": "2025-02-03T00:00:00Z"
    }
  ],
  "cursor": null
}
```

---

### Vote & Label Service

#### `POST /xrpc/app.changala.ring.registerVote`

Upvote a note or brain node. **Auth required.** One vote per user per subject.

```json
// Request
{ "voteUri": "at://did:plc:user/app.changala.vote/abc", "subjectUri": "at://..." }

// Response (200)
{ "subjectUri": "at://...", "totalVotes": 5 }
```

**Errors:** `InvalidRequest` — already voted

---

#### `POST /xrpc/app.changala.ring.applyLabel`

Apply a quality label to content. **Auth required.**

```json
// Request
{ "subjectUri": "at://...", "val": "endorsed" }

// Response (200)
{ "labelUri": "at://..." }
```

Label values: `"endorsed"`, `"communityVerified"`, `"archived"`

---

#### `POST /xrpc/app.changala.ring.retractLabel`

Retract a previously applied label. Creates a negation record. **Auth required.**

```json
// Request
{ "subjectUri": "at://...", "val": "endorsed" }

// Response (200)
{ "labelUri": "at://..." }
```

---

### Brain Service

#### `POST /xrpc/app.changala.ring.createNode`

Create a new brain node (knowledge entry). **Auth required.**

```json
// Request
{
  "title": "Divide and Conquer",
  "content": "# Divide and Conquer\n\nA fundamental algorithmic paradigm...",
  "format": "markdown",
  "tags": ["algorithms", "paradigms"],
  "academicRef": "at://did:web:ring/app.changala.course/cs301",
  "summary": "Core algorithmic paradigm"
}

// Response (200)
{
  "ringRef": { "ringDid": "did:web:ring.changala.local", "cid": "sha256-..." },
  "nodeTemplate": {
    "uri": "at://did:plc:user/app.changala.brain.node/abc",
    "title": "Divide and Conquer",
    "format": "markdown",
    "ringRef": { "ringDid": "...", "cid": "..." },
    "tags": ["algorithms", "paradigms"],
    "academicRef": "at://...",
    "version": 1,
    "summary": "Core algorithmic paradigm",
    "createdAt": "2025-02-05T00:00:00Z"
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `title` | `string` | ✅ | Node title |
| `content` | `string` | ✅ | Raw content (markdown, latex, etc.) |
| `format` | `string` | ✅ | `"markdown"`, `"plaintext"`, `"latex"`, `"html"` |
| `tags` | `string[]` | ❌ | Free-form tags (max 32) |
| `academicRef` | `string` | ❌ | AT URI linking to a course/session |
| `summary` | `string` | ❌ | Short description (shown in graph tooltips) |
| `createdAt` | `string` | ❌ | ISO 8601 (defaults to server time) |

---

#### `POST /xrpc/app.changala.ring.versionNode`

Create a new version of a brain node. **Auth required.**

| Field | Type | Required |
|---|---|---|
| `parentNodeUri` | `string` | ✅ |
| `title` | `string` | ✅ |
| `content` | `string` | ✅ |
| `format` | `string` | ✅ |
| `tags` | `string[]` | ❌ |
| `academicRef` | `string` | ❌ |
| `summary` | `string` | ❌ |

Response: Same as `createNode`, with `parentNode` and incremented `version`.

---

#### `GET /xrpc/app.changala.ring.getNodeContent`

Fetch raw brain node content from blob store. No auth required.

| Param | Type | Required |
|---|---|---|
| `cid` | `string` | ✅ |

```json
{ "format": "markdown", "content": "# Divide and Conquer\n..." }
```

---

#### `POST /xrpc/app.changala.ring.createLink`

Create a directed link between two nodes. **Auth required.**

```json
// Request
{ "fromUri": "at://...node/a", "toUri": "at://...node/b", "label": "expands" }

// Response (200)
{ "linkUri": "at://did:plc:user/app.changala.brain.link/abc", "createdAt": "..." }
```

**Errors:** `InvalidRequest` — duplicate link

---

#### `POST /xrpc/app.changala.ring.deleteLink`

Delete a brain link. **Auth required.**

```json
{ "linkUri": "at://..." }
// Response: { "linkUri": "at://..." }
```

---

### Moderation Service

#### `POST /xrpc/app.changala.ring.banDid`

Ban a user. **Admin only.**

```json
// Request
{ "targetDid": "did:plc:badactor", "reason": "Spam", "ttlSeconds": 86400 }

// Response (200)
{ "targetDid": "did:plc:badactor", "permanent": false, "expiresAt": "2025-02-02T00:00:00Z", "bannedAt": "2025-02-01T00:00:00Z" }
```

| Field | Type | Required | Description |
|---|---|---|---|
| `targetDid` | `string` | ✅ | DID to ban |
| `reason` | `string` | ❌ | Ban reason |
| `ttlSeconds` | `integer` | ❌ | Ban duration in seconds. Omit for permanent ban. |

---

#### `POST /xrpc/app.changala.ring.liftBan`

Remove a ban. **Admin only.**

```json
{ "targetDid": "did:plc:badactor" }
// Response: { "targetDid": "...", "liftedAt": "..." }
```

---

#### `GET /xrpc/app.changala.ring.listBans`

List active bans. **Admin only.** Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `permanentOnly` | `boolean` | ❌ | `false` |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

---

#### `GET /xrpc/app.changala.ring.isBanned`

Check ban status. No auth required.

| Param | Type | Required |
|---|---|---|
| `did` | `string` | ✅ |

```json
{ "banned": true, "expiresAt": "2025-02-02T00:00:00Z" }
// or
{ "banned": false, "expiresAt": null }
```

---

### Archive Service

#### `POST /xrpc/app.changala.ring.initiateArchive`

Begin archival for a course+semester. **Admin only.**

```json
// Request
{ "courseUri": "at://...", "semester": "Fall 2025" }

// Response (200)
{ "courseUri": "at://...", "semester": "Fall 2025", "initiatedAt": "...", "sessionCount": 12, "noteCount": 47 }
```

---

#### `POST /xrpc/app.changala.ring.sealArchive`

Seal an initiated archive as immutable. **Admin only.**

```json
// Request
{ "courseUri": "at://...", "semester": "Fall 2025" }

// Response (200)
{ "archiveUri": "at://...", "bundleRef": { "ringDid": "...", "cid": "..." }, "sealedAt": "...", "sessionCount": 12, "noteCount": 47 }
```

---

#### `POST /xrpc/app.changala.ring.exportArchive`

Export a sealed archive. **Auth required.**

```json
{ "courseUri": "at://...", "semester": "Fall 2025", "format": "latex" }
// Response: { "exportRef": {...}, "format": "latex", "sizeBytes": 0, "createdAt": "..." }
```

---

#### `POST /xrpc/app.changala.ring.uploadToInternetArchive`

Upload sealed archive to archive.org. **Admin only.** MVP: generates fake IA identifier.

```json
{ "courseUri": "at://...", "semester": "Fall 2025" }
// Response: { "iaIdentifier": "changala-...", "iaUrl": "https://archive.org/details/...", "uploadedAt": "..." }
```

---

#### `GET /xrpc/app.changala.ring.getArchive`

Get archive metadata. No auth required.

| Param | Type | Required |
|---|---|---|
| `courseUri` | `string` | ✅ |
| `semester` | `string` | ✅ |

```json
{
  "archiveUri": "at://...",
  "courseUri": "at://...",
  "semester": "Fall 2025",
  "sealedBy": "did:plc:admin",
  "bundleRef": { "ringDid": "...", "cid": "..." },
  "internetArchiveUrl": "https://archive.org/details/...",
  "sessionCount": 12,
  "noteCount": 47,
  "sealedAt": "2025-06-15T00:00:00Z"
}
```

---

## Global View Endpoints

### Feed Service

#### `GET /xrpc/app.changala.globalview.getNotes`

Get notes for a session with vote counts and labels. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `sessionUri` | `string` | ✅ | — |
| `sortBy` | `string` | ❌ | `"createdAt"` (`"votes"` or `"createdAt"`) |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "notes": [
    {
      "uri": "at://did:plc:user/app.changala.note/abc",
      "authorDid": "did:plc:user",
      "sessionUri": "at://...",
      "format": "latex",
      "ringRef": { "ringDid": "...", "cid": "..." },
      "version": 1,
      "voteCount": 5,
      "labels": [
        { "val": "endorsed", "srcDid": "did:plc:faculty", "createdAt": "..." }
      ],
      "summary": "Sorting algorithm notes",
      "createdAt": "2025-02-01T11:30:00Z"
    }
  ],
  "cursor": null
}
```

---

#### `GET /xrpc/app.changala.globalview.getCourseFeed`

Unified chronological feed of all activity for a course. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `courseUri` | `string` | ✅ | — |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "feed": [
    {
      "eventType": "noteCreated",
      "actorDid": "did:plc:student",
      "subjectUri": "at://...note/abc",
      "sessionUri": "at://...session/xyz",
      "createdAt": "2025-02-01T11:30:00Z"
    }
  ],
  "cursor": null
}
```

Event types: `"sessionOpened"`, `"sessionEnded"`, `"sessionCancelled"`, `"sessionRescheduled"`, `"sessionScheduled"`, `"noteCreated"`

---

#### `GET /xrpc/app.changala.globalview.getSocialFeed`

Recent activity with mode switch. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `mode` | `string` | ❌ | `"all"` (`"academic"`, `"brain"`, `"all"`) |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "feed": [
    {
      "eventType": "noteCreated",
      "actorDid": "did:plc:user",
      "subjectUri": "at://...",
      "courseUri": "at://...",
      "sessionUri": "at://...",
      "createdAt": "..."
    }
  ],
  "cursor": null
}
```

---

#### `GET /xrpc/app.changala.globalview.getBrainFeed`

Recent brain nodes with vote counts. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "nodes": [
    {
      "uri": "at://did:plc:user/app.changala.brain.node/abc",
      "authorDid": "did:plc:user",
      "title": "Divide and Conquer",
      "format": "markdown",
      "ringRef": { "ringDid": "...", "cid": "..." },
      "tags": ["algorithms", "paradigms"],
      "academicRef": "at://...",
      "voteCount": 3,
      "summary": "Core paradigm",
      "createdAt": "2025-02-05T00:00:00Z"
    }
  ],
  "cursor": null
}
```

---

#### `GET /xrpc/app.changala.globalview.getKeywordHistogram`

Keyword frequency histogram for a session. No auth required.

| Param | Type | Required |
|---|---|---|
| `sessionUri` | `string` | ✅ |

```json
{
  "sessionUri": "at://...",
  "entries": [
    { "text": "quicksort", "count": 12 },
    { "text": "mergesort", "count": 8 }
  ],
  "totalSubmissions": 25,
  "windowOpen": false,
  "windowExpiresAt": "2025-02-01T12:00:00Z"
}
```

---

#### `GET /xrpc/app.changala.globalview.getTrendingKeywords`

Most popular keywords across all active sessions. No auth required.

| Param | Type | Required | Default |
|---|---|---|---|
| `withinHours` | `integer` | ❌ | 24 |
| `limit` | `integer` | ❌ | 20 |

```json
{
  "keywords": [
    { "text": "quicksort", "count": 42, "sessionUri": "at://...", "courseUri": "at://..." }
  ],
  "computedAt": "2025-02-01T12:00:00Z"
}
```

---

#### `GET /xrpc/app.changala.globalview.getTrendingBrainTags`

Most popular brain node tags. No auth required.

| Param | Type | Required | Default |
|---|---|---|---|
| `withinDays` | `integer` | ❌ | 7 |
| `limit` | `integer` | ❌ | 20 |

```json
{
  "tags": [
    { "tag": "algorithms", "count": 15, "sampleNodeUri": "at://..." }
  ],
  "computedAt": "2025-02-01T12:00:00Z"
}
```

---

#### `GET /xrpc/app.changala.globalview.getFollowedEnrollments`

MVP: returns most popular courses by enrollment count (ATProto social graph not yet integrated).

| Param | Type | Required | Default |
|---|---|---|---|
| `limit` | `integer` | ❌ | 20 |

```json
{
  "enrollments": [
    { "courseUri": "at://...", "courseTitle": "Intro to Algorithms", "institutionDid": "did:web:...", "followedCount": 42 }
  ]
}
```

---

#### `GET /xrpc/app.changala.globalview.getGlobalArchiveFeed`

Recently sealed archives. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `institutionDid` | `string` | ❌ | — |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "items": [
    {
      "archiveUri": "at://...",
      "courseUri": "at://...",
      "courseTitle": "Intro to Algorithms",
      "semester": "Fall 2025",
      "institutionDid": "did:web:...",
      "iaUrl": "https://archive.org/details/...",
      "sessionCount": 12,
      "sealedAt": "2025-06-15T00:00:00Z"
    }
  ],
  "cursor": null
}
```

---

### Search Service

All search endpoints use PostgreSQL `ILIKE` for MVP. No auth required. Paginated.

#### `GET /xrpc/app.changala.globalview.searchNotes`

| Param | Type | Required | Default |
|---|---|---|---|
| `q` | `string` | ✅ | — |
| `courseUri` | `string` | ❌ | — |
| `semester` | `string` | ❌ | — |
| `archivedOnly` | `boolean` | ❌ | `false` |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{ "notes": [/* NoteView objects */], "hitsTotal": 42, "cursor": null }
```

---

#### `GET /xrpc/app.changala.globalview.searchCourses`

| Param | Type | Required |
|---|---|---|
| `q` | `string` | ✅ |
| `semester` | `string` | ❌ |
| `limit` | `integer` | ❌ |
| `cursor` | `string` | ❌ |

```json
{
  "courses": [
    { "uri": "at://...", "title": "...", "code": "CS301", "department": "CS", "semester": "Fall 2025", "visibility": "institution", "enrolledCount": 42, "institutionDid": "did:web:..." }
  ],
  "hitsTotal": 5,
  "cursor": null
}
```

---

#### `GET /xrpc/app.changala.globalview.searchArchive`

| Param | Type | Required |
|---|---|---|
| `q` | `string` | ✅ |
| `semester` | `string` | ❌ |
| `limit` | `integer` | ❌ |
| `cursor` | `string` | ❌ |

```json
{ "results": [/* ArchiveFeedItem objects */], "hitsTotal": 3, "cursor": null }
```

---

#### `GET /xrpc/app.changala.globalview.searchBrainNodes`

| Param | Type | Required |
|---|---|---|
| `q` | `string` | ✅ |
| `authorDid` | `string` | ❌ |
| `tags` | `string[]` | ❌ |
| `limit` | `integer` | ❌ |
| `cursor` | `string` | ❌ |

```json
{ "nodes": [/* BrainNodeView objects */], "hitsTotal": 12, "cursor": null }
```

---

### Graph Service

#### `GET /xrpc/app.changala.globalview.getNodeGraph`

BFS-based local subgraph traversal. Returns nodes and edges for rendering with d3-force or cytoscape.js. No auth required.

| Param | Type | Required | Default | Max |
|---|---|---|---|---|
| `nodeUri` | `string` | ✅ | — | — |
| `depth` | `integer` | ❌ | 2 | 3 |

```json
{
  "nodes": [
    { "uri": "at://...", "title": "Divide and Conquer", "authorDid": "did:plc:user", "tags": ["algorithms"], "summary": "..." }
  ],
  "edges": [
    { "fromUri": "at://...node/a", "toUri": "at://...node/b", "label": "expands" }
  ]
}
```

Max 100 nodes per response. BFS traverses both outbound and inbound links.

---

#### `GET /xrpc/app.changala.globalview.getBacklinks`

All nodes that link TO a given node. No auth required. Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `nodeUri` | `string` | ✅ | — |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "backlinks": [
    {
      "fromUri": "at://...node/other",
      "fromTitle": "Recursion Patterns",
      "fromAuthorDid": "did:plc:user2",
      "label": "relates to",
      "createdAt": "2025-02-06T01:00:00Z"
    }
  ],
  "cursor": null
}
```

---

#### `GET /xrpc/app.changala.globalview.getNeighbours`

Flat list of nodes within N hops. No auth required.

| Param | Type | Required | Default | Max |
|---|---|---|---|---|
| `nodeUri` | `string` | ✅ | — | — |
| `hops` | `integer` | ❌ | 1 | 3 |
| `limit` | `integer` | ❌ | 50 | 100 |

```json
{
  "neighbours": [
    {
      "uri": "at://...node/b",
      "title": "Recursion Patterns",
      "authorDid": "did:plc:user",
      "tags": ["recursion"],
      "distance": 1,
      "summary": "Common recursion patterns"
    }
  ]
}
```

Sorted by distance ASC, then URI.

---

### Notification Service

#### `GET /xrpc/app.changala.globalview.getNotifications`

Get notifications for the authenticated user. **Auth required.** Paginated.

| Param | Type | Required | Default |
|---|---|---|---|
| `unreadOnly` | `boolean` | ❌ | `false` |
| `limit` | `integer` | ❌ | 50 |
| `cursor` | `string` | ❌ | — |

```json
{
  "notifications": [
    {
      "id": "notif-001",
      "recipientDid": "did:plc:user",
      "reason": "vote_received",
      "subjectUri": "at://...note/abc",
      "read": false,
      "createdAt": "2025-02-02T00:00:00Z"
    }
  ],
  "unreadCount": 3,
  "cursor": null
}
```

Notification reasons: `"sessionOpened"`, `"sessionCancelled"`, `"sessionRescheduled"`, `"keywordWindowClosing"`, `"noteVoted"`, `"editProposed"`, `"editAccepted"`, `"archiveInitiated"`, `"labelApplied"`, `"brainNodeLinked"`

---

#### `POST /xrpc/app.changala.globalview.markNotificationRead`

Mark a single notification as read. **Auth required.**

```json
{ "notificationId": "notif-001" }
// Response: { "notificationId": "notif-001" }
```

---

#### `POST /xrpc/app.changala.globalview.markAllRead`

Mark all unread notifications as read. **Auth required.**

```json
{}
// Response: { "markedCount": 3 }
```

---

## Firehose Events

Changala subscribes to the ATProto Jetstream firehose for real-time event materialisation. The following collections are monitored:

| Collection | Event | Action |
|---|---|---|
| `app.changala.keyword` | create | Materialise into `keywords` table |
| `app.changala.vote` | create | Materialise into `votes` table |
| `app.changala.label` | create | Materialise into `labels` table (assertion or negation) |
| `app.changala.brain.node` | create | Upsert into `brain_nodes` table (handles versioning) |
| `app.changala.brain.link` | create | Insert into `brain_links` table (deduplication via ON CONFLICT) |

MVP limitation: only `create` operations are handled. Updates and deletes are deferred.

---

## Env Var Overrides

Every config value can be overridden by an environment variable. Env vars take precedence over `atrg.toml` — use for k8s Secrets, Docker `.env`, ConfigMaps, etc.

There are two sets of env vars:

### Framework Config (`ATRG_*`)

These override the framework-level `[app]`, `[auth]`, and `[database]` sections from `atrg.toml`. Handled by `atrg-core` automatically.

| Environment Variable | Config Path | Format | Example |
|---|---|---|---|
| `ATRG_APP__NAME` | `[app] name` | string | `changala` |
| `ATRG_APP__HOST` | `[app] host` | string | `0.0.0.0` |
| `ATRG_APP__PORT` | `[app] port` | integer | `3000` |
| `ATRG_APP__SECRET_KEY` | `[app] secret_key` | string (≥32 chars) | `openssl rand -hex 32` |
| `ATRG_APP__CORS_ORIGINS` | `[app] cors_origins` | comma-separated | `http://localhost:5173,https://changala.app` or `*` |
| `ATRG_APP__ENVIRONMENT` | `[app] environment` | `development` or `production` | `production` |
| `ATRG_AUTH__CLIENT_ID` | `[auth] client_id` | URL | `https://changala.app/client-metadata.json` |
| `ATRG_AUTH__REDIRECT_URI` | `[auth] redirect_uri` | URL | `https://changala.app/auth/callback` |
| `ATRG_AUTH__SCOPE` | `[auth] scope` | string | `atproto transition:generic` |
| `ATRG_DATABASE__URL` | `[database] url` | connection string | `postgres://user:pass@host:5432/changala` |

### Application Config (`CHANGALA_*`)

These override the `[changala]` section (business-logic database + S3 blob store). Handled by Changala's own `apply_env_overrides()`.

| Environment Variable | Config Path | Example |
|---|---|---|
| `CHANGALA_DATABASE_URL` | `[changala] database_url` | `postgres://user:pass@host:5432/changala` |
| `CHANGALA_S3_ENDPOINT` | `[changala.s3] endpoint` | `http://garage:3900` |
| `CHANGALA_S3_BUCKET` | `[changala.s3] bucket` | `changala-blobs` |
| `CHANGALA_S3_REGION` | `[changala.s3] region` | `us-east-1` |
| `CHANGALA_S3_ACCESS_KEY` | `[changala.s3] access_key` | `GK3a787017baa815a7` |
| `CHANGALA_S3_SECRET_KEY` | `[changala.s3] secret_key` | `abc123...` |
| `CHANGALA_S3_PATH_STYLE` | `[changala.s3] path_style` | `true` (default) or `false` |

### Kubernetes / Docker Quick Reference

```yaml
# k8s Secret (sensitive values)
apiVersion: v1
kind: Secret
metadata:
  name: changala-secrets
stringData:
  ATRG_APP__SECRET_KEY: "<openssl rand -hex 32>"
  CHANGALA_DATABASE_URL: "postgres://changala:pass@postgres:5432/changala"
  CHANGALA_S3_ACCESS_KEY: "GK..."
  CHANGALA_S3_SECRET_KEY: "..."

---
# k8s ConfigMap (non-sensitive values)
apiVersion: v1
kind: ConfigMap
metadata:
  name: changala-config
data:
  ATRG_APP__HOST: "0.0.0.0"
  ATRG_APP__PORT: "3000"
  ATRG_APP__ENVIRONMENT: "production"
  ATRG_APP__CORS_ORIGINS: "https://changala.app"
  ATRG_AUTH__CLIENT_ID: "https://changala.app/client-metadata.json"
  ATRG_AUTH__REDIRECT_URI: "https://changala.app/auth/callback"
  ATRG_DATABASE__URL: "postgres://changala:pass@postgres:5432/changala"
  CHANGALA_S3_ENDPOINT: "http://garage:3900"
  CHANGALA_S3_BUCKET: "changala-blobs"
  CHANGALA_S3_REGION: "us-east-1"
```

On startup, applied overrides are logged:

```
INFO applied env var overrides overrides=["CHANGALA_DATABASE_URL", "CHANGALA_S3_ACCESS_KEY", "CHANGALA_S3_SECRET_KEY"]
```

---

## Endpoint Quick Reference

### Ring (Write) — `POST` unless noted

| Endpoint | Auth | Method |
|---|---|---|
| `app.changala.ring.verifyEmail` | ❌ | POST |
| `app.changala.ring.getMemberships` | ❌ | GET |
| `app.changala.ring.getRole` | ❌ | GET |
| `app.changala.ring.createCourse` | Admin | POST |
| `app.changala.ring.getCourse` | ❌ | GET |
| `app.changala.ring.listCourses` | ❌ | GET |
| `app.changala.ring.enrollStudent` | Auth | POST |
| `app.changala.ring.getEnrollments` | ❌ | GET |
| `app.changala.ring.assignClassRep` | Admin | POST |
| `app.changala.ring.createSession` | ClassRep/Admin | POST |
| `app.changala.ring.getSession` | ❌ | GET |
| `app.changala.ring.listSessions` | ❌ | GET |
| `app.changala.ring.openSession` | ClassRep/Admin | POST |
| `app.changala.ring.closeSession` | ClassRep/Admin | POST |
| `app.changala.ring.cancelSession` | ClassRep/Admin | POST |
| `app.changala.ring.rescheduleSession` | ClassRep/Admin | POST |
| `app.changala.ring.addKeyword` | Auth | POST |
| `app.changala.ring.createNote` | Auth | POST |
| `app.changala.ring.versionNote` | Auth | POST |
| `app.changala.ring.getNoteContent` | ❌ | GET |
| `app.changala.ring.getNoteHistory` | ❌ | GET |
| `app.changala.ring.proposeEdit` | Auth | POST |
| `app.changala.ring.acceptEdit` | ClassRep/Admin | POST |
| `app.changala.ring.rejectEdit` | ClassRep/Admin | POST |
| `app.changala.ring.getCollectiveNote` | ❌ | GET |
| `app.changala.ring.listEditProposals` | ❌ | GET |
| `app.changala.ring.registerVote` | Auth | POST |
| `app.changala.ring.applyLabel` | Auth | POST |
| `app.changala.ring.retractLabel` | Auth | POST |
| `app.changala.ring.createNode` | Auth | POST |
| `app.changala.ring.versionNode` | Auth | POST |
| `app.changala.ring.getNodeContent` | ❌ | GET |
| `app.changala.ring.createLink` | Auth | POST |
| `app.changala.ring.deleteLink` | Auth | POST |
| `app.changala.ring.banDid` | Admin | POST |
| `app.changala.ring.liftBan` | Admin | POST |
| `app.changala.ring.listBans` | Admin | GET |
| `app.changala.ring.isBanned` | ❌ | GET |
| `app.changala.ring.initiateArchive` | Admin | POST |
| `app.changala.ring.sealArchive` | Admin | POST |
| `app.changala.ring.exportArchive` | Auth | POST |
| `app.changala.ring.uploadToInternetArchive` | Admin | POST |
| `app.changala.ring.getArchive` | ❌ | GET |

### Global View (Read) — All `GET` unless noted

| Endpoint | Auth | Method |
|---|---|---|
| `app.changala.globalview.getNotes` | ❌ | GET |
| `app.changala.globalview.getCourseFeed` | ❌ | GET |
| `app.changala.globalview.getSocialFeed` | ❌ | GET |
| `app.changala.globalview.getBrainFeed` | ❌ | GET |
| `app.changala.globalview.getKeywordHistogram` | ❌ | GET |
| `app.changala.globalview.getTrendingKeywords` | ❌ | GET |
| `app.changala.globalview.getTrendingBrainTags` | ❌ | GET |
| `app.changala.globalview.getFollowedEnrollments` | ❌ | GET |
| `app.changala.globalview.getGlobalArchiveFeed` | ❌ | GET |
| `app.changala.globalview.searchNotes` | ❌ | GET |
| `app.changala.globalview.searchCourses` | ❌ | GET |
| `app.changala.globalview.searchArchive` | ❌ | GET |
| `app.changala.globalview.searchBrainNodes` | ❌ | GET |
| `app.changala.globalview.getNodeGraph` | ❌ | GET |
| `app.changala.globalview.getBacklinks` | ❌ | GET |
| `app.changala.globalview.getNeighbours` | ❌ | GET |
| `app.changala.globalview.getNotifications` | Auth | GET |
| `app.changala.globalview.markNotificationRead` | Auth | POST |
| `app.changala.globalview.markAllRead` | Auth | POST |

---

*Generated from implementation — May 2025. 62 XRPC endpoints, 156 typed structs, 9 Postgres migration files.*
