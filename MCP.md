# Changala MCP — AI-Powered Institution Management

> Model Context Protocol server for managing a Changala Ring via AI assistants.
> The MCP server runs directly on the Ring — no separate deployment needed.

---

## Quick Start

### 1. Deploy the Ring with MCP enabled

Add these env vars to your Ring deployment:

```
CHANGALA_MCP_ENABLED=true
CHANGALA_BOOTSTRAP_API_KEY=chg_your-secret-key-here
```

The bootstrap API key is auto-provisioned on startup with `admin:*` scopes.

### 2. Verify MCP is running

```bash
curl https://your-ring.example.com/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}'
```

You should get back a JSON-RPC response with the server's capabilities and tool list.

### 3. Connect your AI client

#### Zed

Add to `~/.config/zed/settings.json`:

```json
{
  "context_servers": {
    "changala": {
      "settings": {
        "url": "https://your-ring.example.com/mcp",
        "headers": {
          "Authorization": "Bearer chg_your-api-key-here"
        }
      }
    }
  }
}
```

#### Claude Desktop

Edit `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "changala": {
      "type": "streamable-http",
      "url": "https://your-ring.example.com/mcp",
      "headers": {
        "Authorization": "Bearer chg_your-api-key-here"
      }
    }
  }
}
```

#### Any MCP-compatible client

The MCP endpoint requires `Authorization: Bearer chg_xxx` on every request.
Without a valid API key, you'll get `401 Unauthorized`.

```
POST https://your-ring.example.com/mcp
Authorization: Bearer chg_your-api-key-here
```

---

## Available Tools

### Course Management

| Tool | Description | Parameters |
|---|---|---|
| `create_course` | Create a new course | `title`, `code`, `department`, `semester`, `description?` |
| `list_courses` | List courses with optional filters | `semester?`, `department?` |
| `enroll_student` | Enroll a student by DID | `course_uri`, `target_did` |
| `assign_class_rep` | Assign class representative | `course_uri`, `class_rep_did` |

### Session Management

| Tool | Description | Parameters |
|---|---|---|
| `create_session` | Schedule a new session | `course_uri`, `scheduled_at`, `duration_mins`, `topic?` |
| `open_session` | Transition session to live | `session_uri` |
| `close_session` | End session, open keyword window | `session_uri` |

### User & Role Management

| Tool | Description | Parameters |
|---|---|---|
| `promote_role` | Change a user's role | `target_did`, `role` (student/classRep/admin) |
| `get_memberships` | View user's memberships | `did` |
| `ban_user` | Ban a user | `target_did`, `reason?` |
| `list_bans` | View active bans | — |

### Admin

| Tool | Description | Parameters |
|---|---|---|
| `get_audit_log` | View recent admin actions | — |
| `list_api_keys` | View API keys (prefix only) | — |

---

## Example Conversations

### Bootstrap a new semester

> **You:** Create these courses for Fall 2026 in the Computer Science department:
> CS301 Introduction to Algorithms, CS302 Data Structures, CS303 Operating Systems

The AI will call `create_course` three times with the appropriate parameters.

### Bulk enrollment

> **You:** Enroll these students into CS301:
> did:plc:student1, did:plc:student2, did:plc:student3

The AI will call `enroll_student` for each DID.

### Session management

> **You:** Open today's CS301 session and close yesterday's

The AI will call `list_courses` to find CS301, then use `open_session` and `close_session`.

### Role management

> **You:** Make did:plc:xyz the class rep for CS301

The AI will call `assign_class_rep` with the course URI and DID.

---

## Architecture

```
┌──────────────────────────────────────┐
│  changala-ring pod                   │
│  port 3000                           │
│                                      │
│  /xrpc/*  → Ring XRPC endpoints     │
│  /mcp     → MCP server (SSE)        │ ← AI clients
│  /auth/*  → OAuth flow              │
│                                      │
│  MCP calls Ring XRPC via loopback:   │
│  http://127.0.0.1:3000/xrpc/...     │
│  Bearer: chg_bootstrap_key          │
└──────────────────────────────────────┘
```

The MCP server runs **inside** the Ring process. When an AI client calls a tool,
the MCP server translates it into an XRPC HTTP request to the Ring's own
endpoints via `127.0.0.1` loopback. Authentication is via the bootstrap API key.

---

## Env Vars

| Variable | Required | Default | Description |
|---|---|---|---|
| `CHANGALA_MCP_ENABLED` | Yes | `false` | Set to `true` to mount MCP at `/mcp` |
| `CHANGALA_BOOTSTRAP_API_KEY` | Yes | — | API key for MCP → Ring auth (also provisions admin key on startup) |
| `CHANGALA_API_KEY` | No | Falls back to bootstrap key | Explicit MCP API key (overrides bootstrap) |
| `CHANGALA_RING_URL` | No | `http://127.0.0.1:{port}` | Ring URL for MCP calls (auto-derived when running inside Ring) |

---

## API Key Management

### Create a key (via XRPC)

```bash
curl -X POST https://your-ring.example.com/xrpc/app.changala.ring.createApiKey \
  -H "Authorization: Bearer <session-token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "MCP Admin Key", "scopes": ["admin:*"]}'
```

Response (key shown ONCE, never retrievable again):

```json
{
  "key": "chg_abc123...",
  "prefix": "chg_abc12345",
  "name": "MCP Admin Key",
  "scopes": ["admin:*"]
}
```

### List keys

```bash
curl https://your-ring.example.com/xrpc/app.changala.ring.listApiKeys \
  -H "Authorization: Bearer <session-token>"
```

### Revoke a key

```bash
curl -X POST https://your-ring.example.com/xrpc/app.changala.ring.revokeApiKey \
  -H "Authorization: Bearer <session-token>" \
  -H "Content-Type: application/json" \
  -d '{"prefix": "chg_abc12345"}'
```

### Bootstrap key (env var)

Set `CHANGALA_BOOTSTRAP_API_KEY=chg_your-secret` on the Ring. It's auto-provisioned
on startup with `admin:*` scopes — no XRPC call needed.

---

## Scopes

| Scope | Permissions |
|---|---|
| `admin:*` | All admin operations |
| `admin:courses` | Create/list courses, assign class reps |
| `admin:sessions` | Session lifecycle |
| `admin:moderation` | Ban/unban users |
| `admin:roles` | Promote/demote users |
| `write:notes` | Create/version notes |
| `write:brain` | Create/version brain nodes and links |
| `read:*` | Read any public data |

---

## Standalone Deployment (Optional)

If you prefer running the MCP server separately from the Ring:

```bash
CHANGALA_API_KEY=chg_xxx \
CHANGALA_RING_URL=https://your-ring.example.com \
MCP_HOST=0.0.0.0 \
MCP_PORT=3001 \
changala-mcp
```

This starts a standalone HTTP server at `0.0.0.0:3001/mcp` that proxies
tool calls to the Ring via XRPC.
