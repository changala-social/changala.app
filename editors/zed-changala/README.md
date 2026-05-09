# Changala — Zed Extension

MCP server extension for [Changala](https://changala.app), a federated social
learning platform built on the AT Protocol.

## Setup

1. Install this extension from Zed's extension marketplace.
2. Add your Ring's MCP URL and access key to Zed settings:

```json
"context_servers": {
  "changala": {
    "url": "https://your-ring.example.com/mcp",
    "headers": {
      "Authorization": "Bearer <your-mcp-access-key>"
    }
  }
}
```

3. Open the Agent Panel — Changala tools will appear automatically.

## Available Tools

| Tool | Description |
|------|-------------|
| `create_course` | Create a new course |
| `list_courses` | List courses (filterable by semester/department) |
| `enroll_student` | Enroll a student by DID |
| `assign_class_rep` | Assign a class representative |
| `create_session` | Create a class session |
| `open_session` | Open a session (transition to live) |
| `close_session` | Close a session (opens keyword window) |
| `ban_user` | Ban a user by DID |
| `list_bans` | List active bans |
| `promote_role` | Change a user's role |
| `get_audit_log` | View admin audit log |
| `get_memberships` | Get user memberships |
| `list_api_keys` | List API keys |

## Security

The MCP endpoint requires a `Bearer` token set via `CHANGALA_MCP_ACCESS_KEY`
on the Ring. Never share your access key publicly.
