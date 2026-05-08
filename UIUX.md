# Changala — Frontend Specification

> UI/UX guide for building the Changala web client.
> Stack: Vite + React + TypeScript. Static deploy to Cloudflare Pages.

---

## Stack

| Layer | Choice | Why |
|---|---|---|
| Build | Vite | Fast, static output for Cloudflare Pages |
| UI | React + TypeScript | AT Protocol ecosystem is React (atcute, atproto-ui) |
| Styling | Tailwind CSS | Utility-first, no runtime |
| AT Protocol | `atrg` built-in OAuth | Server-side PKCE + DPoP via `/auth/*` routes — no client-side AT Protocol libs needed |
| AT Protocol UI | `atproto-ui` | Optional — `useDidResolution` for resolving DIDs to @handles |
| LaTeX | KaTeX | Fast client-side rendering, static-compatible |
| Markdown | `react-markdown` + `remark-gfm` | Renders brain node content |
| Graph | `d3-force` or `cytoscape.js` | Brain node graph visualization |
| State | `@tanstack/react-query` | Cache, dedup, paginate XRPC calls |

---

## Two Modes

The app has one global toggle. All feeds and searches pass `mode` to the API.

| Mode | What shows | API param |
|---|---|---|
| **Academic** | Courses, sessions, notes, keywords, archives | `mode=academic` |
| **Brain** | Knowledge nodes, backlinks, tags, graph | `mode=brain` |
| **All** (default) | Everything | `mode=all` |

The toggle is a UI switch in the header. It sets a query param / React state that flows into every feed and search call.

---

## Pages

### Public (no auth)

| Route | Page | API calls |
|---|---|---|
| `/` | Landing + course search | `searchCourses`, `getTrendingKeywords`, `getTrendingBrainTags` |
| `/courses` | Course catalog | `listCourses` (paginated, filterable by semester/department) |
| `/course/:uri` | Course detail | `getCourse`, `listSessions`, `getCourseFeed` |
| `/session/:uri` | Session detail | `getSession`, `getNotes`, `getKeywordHistogram` |
| `/note/:cid` | Note viewer | `getNoteContent` (renders LaTeX/markdown) |
| `/brain` | Brain feed | `getBrainFeed`, `getTrendingBrainTags` |
| `/brain/:uri` | Single node | `getNodeContent`, `getBacklinks`, `getNodeGraph` |
| `/graph/:uri` | Graph explorer | `getNodeGraph` (d3-force/cytoscape) |
| `/archive` | Archive browser | `getGlobalArchiveFeed`, `searchArchive` |
| `/search` | Unified search | `searchNotes`, `searchCourses`, `searchBrainNodes` |
| `/profile/:did` | Public profile | `getMemberships`, brain nodes by author |

### Authenticated

| Route | Page | API calls |
|---|---|---|
| `/login` | AT Protocol OAuth flow | `@atcute/client` OAuth |
| `/dashboard` | My courses + notifications | `getNotifications`, `getFollowedEnrollments` |
| `/session/:uri/live` | Live session | `addKeyword`, `getKeywordHistogram` (polling) |
| `/note/new` | Note editor | `createNote` |
| `/brain/new` | Brain node editor | `createNode`, `createLink` |
| `/brain/:uri/edit` | Edit brain node | `versionNode` |
| `/admin` | Admin panel | `createCourse`, `assignClassRep`, `banDid`, `initiateArchive` |

---

## Components

### Layout

```
┌──────────────────────────────────────────────┐
│  Header                                      │
│  [Logo] [Search] [Mode Toggle] [Auth/Avatar] │
├──────────────────────────────────────────────┤
│                                              │
│  <Page Content>                              │
│                                              │
└──────────────────────────────────────────────┘
```

- **Header**: Always visible. Contains mode toggle (Academic / Brain / All), search bar, auth button.
- **No sidebar**. Mobile-first, single column.
- **Mode toggle**: pill button — `[Academic | Brain | All]`. Persisted to `localStorage`.

### Core Components

#### `<ModeToggle />`
Three-way pill toggle. Sets `mode` in React context. Every feed/search component reads from this context.

#### `<CourseCard />`
Displays: title, code, department, semester, enrollment count, class rep badge.

#### `<SessionCard />`
Displays: topic, status badge (color-coded), scheduled time, keyword window indicator.
- Status colors: `scheduled` = gray, `live` = green pulse, `ended` = blue, `cancelled` = red, `rescheduled` = orange.

#### `<NoteView />`
Renders note content based on `format`:
- `latex` → KaTeX render (`katex.renderToString` or `<InlineMath>` / `<BlockMath>` from `react-katex`)
- `markdown` → `react-markdown` with `remark-gfm`
- `plaintext` → `<pre>`
- `html` → sanitized HTML (`DOMPurify`)

Shows: vote count, labels, author DID (resolved to handle via `atproto-ui`'s `useDidResolution`).

#### `<KeywordHistogram />`
Horizontal bar chart. Entries sorted by count descending.
- Live polling: if `windowOpen === true`, poll `getKeywordHistogram` every 5 seconds.
- Shows `totalSubmissions` and time remaining if window is open.

#### `<KeywordInput />`
Text input + submit button. Calls `addKeyword`. Only visible when `windowOpen === true`.

#### `<BrainNodeCard />`
Displays: title, tags as pills, summary, vote count, format badge, `academicRef` link if present.

#### `<GraphView />`
Full-page or panel graph visualization using `getNodeGraph`.
- Nodes: circles with title labels
- Edges: lines with optional label
- Click node → navigate to `/brain/:uri`
- Use `d3-force` for layout. Feed it `nodes` and `edges` arrays directly from the API response.

#### `<BacklinkList />`
List of nodes that link TO the current node. From `getBacklinks`. Each item shows: `fromTitle`, `fromAuthorDid`, `label`, link to the source node.

#### `<NotificationBell />`
Badge with `unreadCount`. Dropdown shows recent notifications. Each notification links to `subjectUri`.

#### `<CollectiveNote />`
Shows the current collective note content + contributor list. Edit proposals listed below with accept/reject buttons (for class rep/admin).

#### `<ArchiveCard />`
Displays: course title, semester, session count, note count, sealed date. Link to `iaUrl` if available.

---

## Content Rendering

### LaTeX

Use [KaTeX](https://katex.org/). It's fast, renders to static HTML, works in Cloudflare Pages (no server needed).

```tsx
import katex from 'katex';
import 'katex/dist/katex.min.css';

function LatexRenderer({ content }: { content: string }) {
  const html = katex.renderToString(content, {
    throwOnError: false,
    displayMode: true,
  });
  return <div dangerouslySetInnerHTML={{ __html: html }} />;
}
```

For mixed content (text with inline `$...$` and display `$$...$$`), use `react-katex`:

```tsx
import { InlineMath, BlockMath } from 'react-katex';
```

Or for full LaTeX documents, parse sections and render each block.

### Markdown

```tsx
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

function MarkdownRenderer({ content }: { content: string }) {
  return <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>;
}
```

### Format Dispatcher

```tsx
function ContentRenderer({ format, content }: { format: string; content: string }) {
  switch (format) {
    case 'latex':  return <LatexRenderer content={content} />;
    case 'markdown': return <MarkdownRenderer content={content} />;
    case 'html': return <div dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(content) }} />;
    case 'plaintext': default: return <pre className="whitespace-pre-wrap">{content}</pre>;
  }
}
```

---

## AT Protocol Integration

### Auth Flow

`atrg` handles the entire OAuth flow server-side. The frontend just redirects and receives a session token. No AT Protocol client libraries needed.

```
1. User clicks "Login with AT Protocol"
2. Frontend redirects to:  GET /auth/login?handle=user.bsky.social
3. atrg redirects user to their PDS authorization page
4. User approves → PDS redirects to /auth/callback
5. atrg exchanges code for tokens, creates a session
6. atrg redirects to frontend with session cookie / token
7. Frontend stores token, attaches to all subsequent XRPC calls
```

The `Authorization: Bearer <token>` header is all the frontend needs. Token refresh is handled by atrg internally.

### DID Resolution

Use `atproto-ui`'s `useDidResolution` hook to resolve DIDs to handles for display:

```tsx
import { useDidResolution } from 'atproto-ui';

function AuthorName({ did }: { did: string }) {
  const { handle, loading } = useDidResolution(did);
  if (loading) return <span className="animate-pulse">...</span>;
  return <span>@{handle}</span>;
}
```

### XRPC Client

Simple `fetch` wrapper. No SDK needed — XRPC is just HTTP.

```tsx
const BASE = 'https://your-ring.example.com';

async function xrpcGet<T>(nsid: string, params?: Record<string, string>): Promise<T> {
  const url = new URL(`/xrpc/${nsid}`, BASE);
  if (params) Object.entries(params).forEach(([k, v]) => url.searchParams.set(k, v));
  const res = await fetch(url, { headers: authHeaders() });
  if (!res.ok) throw await res.json();
  return res.json();
}

async function xrpcPost<T>(nsid: string, body: unknown): Promise<T> {
  const res = await fetch(`${BASE}/xrpc/${nsid}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw await res.json();
  return res.json();
}

function authHeaders(): Record<string, string> {
  const token = localStorage.getItem('changala_access_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}
```

### Using with react-query

```tsx
import { useQuery } from '@tanstack/react-query';

function useCourses(semester?: string) {
  return useQuery({
    queryKey: ['courses', semester],
    queryFn: () => xrpcGet('app.changala.ring.listCourses', semester ? { semester } : {}),
  });
}

function useKeywordHistogram(sessionUri: string) {
  return useQuery({
    queryKey: ['histogram', sessionUri],
    queryFn: () => xrpcGet('app.changala.globalview.getKeywordHistogram', { sessionUri }),
    refetchInterval: 5000, // poll every 5s for live sessions
  });
}
```

---

## Graph Visualization

The brain graph is the killer feature. Use `d3-force` for the interactive layout.

### Data Shape (from `getNodeGraph`)

```json
{
  "nodes": [{ "uri": "at://...", "title": "Divide and Conquer", "authorDid": "...", "tags": [...], "summary": "..." }],
  "edges": [{ "fromUri": "at://...", "toUri": "at://...", "label": "expands" }]
}
```

### d3-force Setup

```tsx
import * as d3 from 'd3';

function GraphView({ nodeUri }: { nodeUri: string }) {
  const { data } = useQuery({
    queryKey: ['graph', nodeUri],
    queryFn: () => xrpcGet('app.changala.globalview.getNodeGraph', { nodeUri, depth: '2' }),
  });

  useEffect(() => {
    if (!data) return;
    const sim = d3.forceSimulation(data.nodes)
      .force('link', d3.forceLink(data.edges).id(d => d.uri).source(d => d.fromUri).target(d => d.toUri))
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width / 2, height / 2));
    // ... render SVG nodes + edges ...
  }, [data]);
}
```

- Center node (the queried one) is highlighted
- Click a node → navigate to `/brain/:uri` or expand the graph
- Hover shows `summary` tooltip
- Tags shown as colored dots on nodes
- Edge labels shown on hover

---

## Theming

Use `atproto-ui` CSS variables as the base. Extend with Changala-specific tokens:

```css
:root {
  /* atproto-ui base */
  --atproto-color-bg: #ffffff;
  --atproto-color-text: #1a1a1a;
  --atproto-color-link: #0066cc;

  /* Changala additions */
  --changala-academic: #2563eb;    /* blue — academic mode accent */
  --changala-brain: #7c3aed;      /* purple — brain mode accent */
  --changala-live: #16a34a;       /* green — live session indicator */
  --changala-cancelled: #dc2626;  /* red — cancelled session */
}

[data-theme="dark"] {
  --atproto-color-bg: #0a0a0a;
  --atproto-color-text: #e5e5e5;
}
```

Mode toggle sets `data-mode="academic"` or `data-mode="brain"` on `<body>`, which swaps the accent color:

```css
[data-mode="academic"] { --changala-accent: var(--changala-academic); }
[data-mode="brain"]    { --changala-accent: var(--changala-brain); }
```

---

## Deployment

Static build → Cloudflare Pages.

```bash
npm run build   # vite build → dist/
# Deploy dist/ to Cloudflare Pages
```

The app is fully static. All data comes from XRPC calls to the Ring server. No server-side rendering needed.

### CORS

The Ring server must allow the frontend origin in `cors_origins`:

```toml
# atrg.toml
[app]
cors_origins = ["https://changala.app"]
```

### `client-metadata.json`

For AT Protocol OAuth, serve a `client-metadata.json` at the app's root:

```json
{
  "client_id": "https://changala.app/client-metadata.json",
  "client_name": "Changala",
  "client_uri": "https://changala.app",
  "redirect_uris": ["https://changala.app/auth/callback"],
  "grant_types": ["authorization_code", "refresh_token"],
  "response_types": ["code"],
  "scope": "atproto transition:generic",
  "token_endpoint_auth_method": "none",
  "application_type": "web",
  "dpop_bound_access_tokens": true
}
```

This is a static JSON file in your `public/` directory. No server needed.

---

## Key Interactions

### Live Keyword Session

1. Student opens `/session/:uri/live`
2. `getSession` → check `keywordWindowOpen`
3. If open: show `<KeywordInput />` + `<KeywordHistogram />` (polling every 5s)
4. Student types keyword → `addKeyword` → histogram updates on next poll
5. When window closes, input disappears, histogram becomes static

### Brain Node Creation

1. Student opens `/brain/new`
2. Editor with format selector (markdown/latex/plaintext)
3. Live preview pane (KaTeX for latex, react-markdown for markdown)
4. Tag input (pills with autocomplete from `getTrendingBrainTags`)
5. Optional `academicRef` picker (search courses/sessions)
6. Submit → `createNode` → redirect to `/brain/:uri`
7. Optionally create links → `createLink` with `fromUri` of the new node

### Vote Flow

1. Show vote button with count on notes and brain nodes
2. Click → `registerVote` → increment count locally (optimistic update)
3. If `InvalidRequest` (already voted), revert

---

*Keep it simple. Ship the course feed first, then the keyword histogram, then the graph view. That's the order that proves the concept.*
