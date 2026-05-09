# Changala Web — Frontend MVP Roadmap

> Vite + React + TypeScript → static build → Cloudflare Pages.
> All data via XRPC calls to Ring + Global View. No SSR.

---

## Current State

| Item | Status |
|---|---|
| Scaffold | ✅ `create-vite` with `react-ts` template |
| Build | ✅ `npm run build` → `dist/` (static) |
| React | 19.x |
| TypeScript | 6.x |
| Vite | 8.x |
| Backend handlers | 62/62 implemented (all Ring + Global View endpoints) |

---

## Milestone 0: Project Foundation

> **Goal:** Dev tooling, routing, XRPC client, auth skeleton, layout shell.
> **Depends on:** Nothing. Pure frontend scaffolding.
> **Deliverable:** An app that boots, routes, and can talk to the Ring.

### 0.1 Install Core Dependencies

```
npm install @tanstack/react-query react-router-dom
npm install -D tailwindcss @tailwindcss/vite
```

### 0.2 Tailwind Setup

- Add `@tailwindcss/vite` plugin to `vite.config.ts`
- Replace `index.css` with Tailwind directives (`@import "tailwindcss"`)
- Add Changala CSS custom properties (academic blue, brain purple, live green, etc.)
- Dark mode support via `class` strategy + `localStorage` toggle

### 0.3 TypeScript Types from Lexicons

Write a codegen script (`scripts/lexicon-to-ts.ts`) that reads `../lexicons/app/changala/**/*.json` and emits TypeScript interfaces into `src/generated/types.ts`.

This is the frontend equivalent of what `atrg generate` does for Rust. One source of truth.

Types to generate:
- All record types (`Membership`, `Course`, `Session`, `Keyword`, `Note`, `Vote`, `Label`, `Archive`, `BrainNode`, `BrainLink`, `CollectiveNoteProposal`)
- All shared defs (`RingRef`, `Role`, `SessionStatus`, `NoteFormat`, `Visibility`, `LabelVal`, `NotificationType`, `LabelSignal`, `Notification`)
- All XRPC request params and response bodies (from procedure/query lexicon input/output schemas)

### 0.4 XRPC Client

Create `src/lib/xrpc.ts`:
- `xrpcGet<T>(nsid, params?)` — GET `/xrpc/{nsid}?params`
- `xrpcPost<T>(nsid, body?)` — POST `/xrpc/{nsid}` with JSON body
- Auth header injection from stored token
- Typed error handling matching the API error shape (`{ error, message }`)
- Base URL configurable via `VITE_RING_URL` and `VITE_GLOBALVIEW_URL` env vars

### 0.5 React Query Provider + Hooks Foundation

- Wrap app in `QueryClientProvider`
- Create `src/hooks/` directory with initial hook pattern:
  - `useXrpcQuery(nsid, params, options)` — generic typed query hook
  - `useXrpcMutation(nsid, options)` — generic typed mutation hook

### 0.6 Routing

Set up `react-router-dom` with the full route table from UIUX.md:

**Public routes:**
| Route | Page Component |
|---|---|
| `/` | `LandingPage` |
| `/courses` | `CourseCatalog` |
| `/course/:uri` | `CourseDetail` |
| `/session/:uri` | `SessionDetail` |
| `/note/:cid` | `NoteViewer` |
| `/brain` | `BrainFeed` |
| `/brain/:uri` | `BrainNodeDetail` |
| `/graph/:uri` | `GraphExplorer` |
| `/archive` | `ArchiveBrowser` |
| `/search` | `SearchPage` |
| `/profile/:did` | `ProfilePage` |

**Authenticated routes:**
| Route | Page Component |
|---|---|
| `/login` | `LoginPage` |
| `/dashboard` | `Dashboard` |
| `/session/:uri/live` | `LiveSession` |
| `/note/new` | `NoteEditor` |
| `/brain/new` | `BrainNodeEditor` |
| `/brain/:uri/edit` | `BrainNodeEdit` |
| `/admin` | `AdminPanel` |

All page components start as stubs returning `<h1>Page Name</h1>`.

### 0.7 Layout Shell

- `<Header />` — Logo, search bar, mode toggle, auth button/avatar
- `<ModeToggle />` — three-way pill (`Academic | Brain | All`), persisted to `localStorage`, provided via React context
- `<ModeContext>` — React context providing `mode` value to all child components
- Single-column, mobile-first layout. No sidebar.

### 0.8 Auth Skeleton

- `<AuthContext>` — stores token, DID, handle. Reads from `localStorage`.
- `<ProtectedRoute>` — wrapper that redirects to `/login` if not authenticated.
- Login page: "Login with AT Protocol" button → redirects to `GET /auth/login?handle=...`
- Callback handler: reads token from redirect, stores in context + `localStorage`
- `client-metadata.json` in `public/` for AT Protocol OAuth

### Milestone 0 Checklist

- [x] Tailwind CSS configured and working
- [x] CSS custom properties for Changala theming (academic/brain/live/cancelled colors)
- [ ] Dark mode toggle (CSS vars exist, UI toggle not yet wired)
- [ ] Lexicon → TypeScript codegen script (types hand-written for now)
- [x] `src/generated/types.ts` generated and importable
- [x] XRPC client (`xrpcGet`, `xrpcPost`) with auth header injection
- [x] React Query provider + generic hooks
- [x] All routes defined, stub pages rendering
- [x] Layout shell with Header, ModeToggle, ModeContext
- [x] Auth context, protected route wrapper, login redirect flow
- [x] `client-metadata.json` in `public/`
- [x] `VITE_RING_URL` / `VITE_GLOBALVIEW_URL` env vars working
- [x] `npm run build` still produces clean static output

---

## Milestone 1: Course Feed (The Academic Core)

> **Goal:** Browse courses, view sessions, see notes. The read-only academic loop.
> **Depends on:** Milestone 0.
> **Deliverable:** A student can find a course, see its sessions, and read notes.

### 1.1 Course Catalog (`/courses`)

- `useCourses(semester?, department?)` hook → `listCourses`
- `<CourseCard />` — title, code, department, semester, enrollment count, class rep badge
- Pagination (cursor-based, "Load more" button)
- Filters: semester dropdown, department dropdown
- Search integration: typing in the header search bar on this page calls `searchCourses`

### 1.2 Course Detail (`/course/:uri`)

- `useCourse(uri)` hook → `getCourse`
- `useCourseFeed(uri)` hook → `getCourseFeed`
- Course header: title, code, department, semester, class rep, enrollment count
- "Enroll" button (authenticated, calls `enrollStudent`)
- Session list (from feed, grouped by date)
- `<SessionCard />` — topic, status badge (color-coded), scheduled time

### 1.3 Session Detail (`/session/:uri`)

- `useSession(uri)` hook → `getSession`
- `useSessionNotes(uri)` hook → `getNotes`
- Session header: topic, status badge, scheduled time, keyword window status
- Notes list: `<NoteCard />` for each note — author, format badge, vote count, summary
- Click note → `/note/:cid`

### 1.4 Note Viewer (`/note/:cid`)

- `useNoteContent(cid)` hook → `getNoteContent`
- `<ContentRenderer />` — dispatches to the correct renderer based on `format`:
  - `latex` → KaTeX (install `katex`, `react-katex`)
  - `markdown` → `react-markdown` + `remark-gfm`
  - `html` → `DOMPurify` sanitized
  - `plaintext` → `<pre>`
- Vote button + count
- Labels display
- Author DID (resolved to handle — stub for now, proper DID resolution in M3)
- Note version history (`getNoteHistory`)

### 1.5 Landing Page (`/`)

- Hero section with Changala description
- Trending keywords (`getTrendingKeywords`) — horizontal tag cloud
- Course search (typeahead → `searchCourses`)
- If in Brain mode: trending brain tags (`getTrendingBrainTags`)

### Install

```
npm install katex react-katex react-markdown remark-gfm dompurify
npm install -D @types/katex @types/dompurify
```

### Milestone 1 Checklist

- [x] `<CourseCard />` component
- [x] `<SessionCard />` component with status color-coding
- [x] `<ContentRenderer />` dispatching to KaTeX / Markdown / HTML / Plaintext
- [x] Course catalog page with pagination + filters
- [x] Course detail page with session list
- [x] Session detail page with notes list
- [x] Note viewer page with content rendering
- [x] Landing page with trending keywords + course search
- [x] Vote button (UI only — wiring in M3)

---

## Milestone 2: Live Session + Keywords

> **Goal:** The real-time academic interaction. Students submit keywords during live sessions.
> **Depends on:** Milestone 1.
> **Deliverable:** A student joins a live session, submits keywords, watches the histogram update.

### 2.1 Keyword Histogram (`<KeywordHistogram />`)

- `useKeywordHistogram(sessionUri)` hook → `getKeywordHistogram`
  - `refetchInterval: 5000` when `windowOpen === true` (live polling)
- Horizontal bar chart — entries sorted by count descending
- Show `totalSubmissions` count
- Countdown timer when keyword window is open (`windowExpiresAt` - now)
- Smooth transitions on count changes (CSS transitions or framer-motion)

### 2.2 Keyword Input (`<KeywordInput />`)

- Text input + submit button
- Only visible when `windowOpen === true`
- Calls `addKeyword` on submit
- Disables after submission (one keyword per student per session)
- Error handling: window closed, already submitted

### 2.3 Live Session Page (`/session/:uri/live`)

- Extends session detail with live features
- Shows `<KeywordHistogram />` prominently
- Shows `<KeywordInput />` when window is open
- Status indicator: pulsing green dot for "Live"
- Auto-redirect from `/session/:uri` when session is live

### Milestone 2 Checklist

- [x] `<KeywordHistogram />` with polling
- [x] `<KeywordInput />` with submission + error handling
- [x] Live session page
- [x] Countdown timer for keyword window
- [x] Status badge pulse animation for live sessions

---

## Milestone 3: Auth, Votes, and Identity

> **Goal:** Full authentication flow. Users can vote, submit keywords for real, create notes.
> **Depends on:** Milestone 0 (auth skeleton), Milestones 1-2 (pages to protect).
> **Deliverable:** Authenticated users can interact with the platform, not just read.

### 3.1 AT Protocol OAuth Flow

- Login page: handle input → redirect to `/auth/login?handle=...`
- Callback handler: parse token from atrg redirect → store in `AuthContext`
- Token refresh handling (atrg manages refresh server-side; frontend just retries on 401)
- Logout: clear token, redirect to `/`

### 3.2 DID Resolution

- Install and configure `atproto-ui` (or write a minimal `useDidResolution` hook)
- `<AuthorName did={did} />` component — resolves DID → `@handle`
- Cache resolved handles in React Query

### 3.3 Vote Wiring

- `useRegisterVote()` mutation → `registerVote`
- Optimistic update: increment count locally, revert on error
- Handle "already voted" error gracefully (show as already-voted state)

### 3.4 Note Creation (`/note/new`)

- Format selector: LaTeX / Markdown / Plaintext
- Text editor (textarea for MVP — CodeMirror in a future milestone)
- Live preview pane (side-by-side):
  - LaTeX: KaTeX preview
  - Markdown: react-markdown preview
- Session picker (which session is this note for?)
- Summary input
- Submit → `createNote` → redirect to `/note/:cid`

### 3.5 Profile Page (`/profile/:did`)

- `useMemberships(did)` hook → `getMemberships`
- Show: handle, institution memberships, role badges
- List of brain nodes by this author (from `getBrainFeed` filtered by author)

### 3.6 Dashboard (`/dashboard`)

- `useNotifications()` hook → `getNotifications`
- `useFollowedEnrollments()` hook → `getFollowedEnrollments`
- Notification list with mark-read actions
- "People you follow" enrollment suggestions
- Quick links to enrolled courses

### 3.7 Notification Bell (`<NotificationBell />`)

- Badge with `unreadCount`
- Dropdown with recent notifications
- Each notification links to `subjectUri`
- Mark individual or all as read

### Milestone 3 Checklist

- [x] Full OAuth login/logout flow working
- [x] DID → handle resolution
- [x] `<AuthorName />` component used everywhere DIDs are displayed
- [x] Vote mutation with optimistic update
- [x] Note creation page with format selector + live preview
- [x] Profile page
- [x] Dashboard with notifications + followed enrollments
- [x] `<NotificationBell />` in header
- [x] Protected routes enforced

---

## Milestone 4: Brain Layer

> **Goal:** The Zettelkasten second brain. Create nodes, link them, browse the graph.
> **Depends on:** Milestone 3 (auth for writes).
> **Deliverable:** A student can create brain nodes, link them, browse other students' brains.

### 4.1 Brain Feed (`/brain`)

- `useBrainFeed(mode)` hook → `getBrainFeed`
- `<BrainNodeCard />` — title, tags (as pills), summary, vote count, format badge, `academicRef` link
- Trending brain tags sidebar/section (`getTrendingBrainTags`)
- Pagination
- Filter by tag (click a tag → filter feed)

### 4.2 Brain Node Detail (`/brain/:uri`)

- `useNodeContent(uri)` hook → `getNodeContent`
- `useBacklinks(uri)` hook → `getBacklinks`
- Content rendered via `<ContentRenderer />`
- Tags displayed as clickable pills
- `academicRef` link (if present) — links to the referenced course/session
- Vote button
- Backlink list: `<BacklinkList />` — shows nodes that link TO this node
- "View in graph" link → `/graph/:uri`

### 4.3 Brain Node Editor (`/brain/new`, `/brain/:uri/edit`)

- Title input
- Format selector (markdown default, also latex/plaintext/html)
- Content editor (textarea for MVP)
- Live preview pane
- Tag input: pill-style with autocomplete from `getTrendingBrainTags`
- Optional `academicRef` picker: search courses/sessions, attach reference
- Link creation: after saving a node, offer to create links (`createLink`)
  - Target picker: search existing brain nodes
  - Optional edge label input
- Submit: `createNode` (new) or `versionNode` (edit) → redirect to `/brain/:uri`

### 4.4 Link Management

- `useCreateLink()` mutation → `createLink`
- `useDeleteLink()` mutation → `deleteLink`
- On brain node detail: "Add link" button → modal with node search + label input
- On brain node detail: existing outbound links shown with delete option (if author)

### Milestone 4 Checklist

- [x] `<BrainNodeCard />` component
- [x] Brain feed page with tag filtering + pagination
- [x] Brain node detail page with content + backlinks
- [x] Brain node editor with live preview + tags + academic ref picker
- [ ] Link creation modal (hooks exist, UI not yet wired on detail page)
- [ ] Link deletion (hooks exist, UI not yet wired on detail page)
- [x] `<BacklinkList />` component

---

## Milestone 5: Graph Visualization

> **Goal:** The killer feature. Interactive graph exploration of the knowledge network.
> **Depends on:** Milestone 4 (brain nodes exist to visualize).
> **Deliverable:** A student can explore the interconnected knowledge graph visually.

### 5.1 Install

```
npm install d3
npm install -D @types/d3
```

### 5.2 Graph View (`/graph/:uri`)

- `useNodeGraph(uri, depth)` hook → `getNodeGraph`
- Full-page `d3-force` simulation:
  - Nodes: circles with title labels
  - Edges: lines with optional label (shown on hover)
  - Center node (queried URI) highlighted with accent color
  - Tags shown as colored dots on nodes
- Interactions:
  - Click node → navigate to `/brain/:uri`
  - Hover node → tooltip with `summary`
  - Hover edge → show edge `label`
  - Drag nodes to rearrange
  - Zoom + pan
  - Depth control slider (1-3 hops, re-fetches `getNodeGraph` with new depth)
- `useNeighbours(uri, maxDistance)` hook → `getNeighbours` for expanded exploration

### 5.3 Mini Graph (Embed)

- Smaller `<MiniGraph />` component for embedding on brain node detail pages
- Shows immediate neighbours only (depth=1)
- Click "Expand" → navigates to full `/graph/:uri`

### 5.4 Mode-Aware Styling

- Academic nodes (have `academicRef`): academic blue accent
- Brain-only nodes: brain purple accent
- Edges from followed users: highlighted / thicker
- Current user's nodes: distinct border

### Milestone 5 Checklist

- [x] `d3-force` graph rendering
- [x] Node click → navigation
- [x] Hover tooltips (summary, edge labels)
- [x] Drag, zoom, pan interactions
- [x] Depth slider with re-fetch
- [x] `<MiniGraph />` embed for brain node detail
- [ ] Mode-aware color coding (basic coloring done, full mode-awareness pending)

---

## Milestone 6: Collective Notes + Admin

> **Goal:** Class rep and admin workflows. Collective notes, moderation, course management.
> **Depends on:** Milestone 3 (auth + roles).
> **Deliverable:** Class reps can manage sessions and collective notes. Admins can manage courses.

### 6.1 Collective Note View

- `useCollectiveNote(sessionUri)` hook → `getCollectiveNote`
- Shows current collective note content + contributor list
- Edit proposal list below (for class rep / admin):
  - `useEditProposals(sessionUri)` hook → `listEditProposals`
  - Each proposal: diff summary, proposer, status, accept/reject buttons

### 6.2 Edit Proposal Flow

- "Propose Edit" button on session page (for enrolled students)
- Diff editor: show current collective note + editable textarea
- Submit → `proposeEdit`
- Class rep view: review proposals, accept/reject with `acceptEdit` / `rejectEdit`

### 6.3 Session Lifecycle (Class Rep)

- On session detail: lifecycle action buttons (visible to class rep / admin only)
  - "Open Session" → `openSession`
  - "Close Session" → `closeSession`
  - "Cancel Session" → `cancelSession` (with reason input)
  - "Reschedule" → `rescheduleSession` (with datetime picker)

### 6.4 Admin Panel (`/admin`)

- **Course Management:**
  - Create course form → `createCourse`
  - Assign class rep → `assignClassRep`
- **Moderation:**
  - Ban DID form → `banDid` (with optional TTL)
  - Active bans list → `listBans`
  - Lift ban → `liftBan`
- **Archive:**
  - Initiate archive → `initiateArchive`
  - Seal archive → `sealArchive`

### Milestone 6 Checklist

- [x] Collective note viewer with contributor list
- [x] Edit proposal list with accept/reject (class rep)
- [ ] Propose edit flow for enrolled students (hooks exist, standalone form not built)
- [ ] Session lifecycle buttons (class rep / admin) (hooks exist, not wired into SessionDetail)
- [x] Admin panel: course creation
- [x] Admin panel: class rep assignment
- [x] Admin panel: moderation (ban/lift/list)
- [x] Admin panel: archive initiation + sealing

---

## Milestone 6.5: RBAC & Admin Provisioning

> **Goal:** Admin bootstrapping from the UI. Role management without DB access.
> **Depends on:** Milestone 3 (auth), Milestone 6 (admin panel).
> **Deliverable:** An admin can provision new admins, promote/demote users, and verify emails from the admin panel.

### 6.5.1 Admin Provisioning Page

- Admin provisioning form: enter DID + shared secret → `provisionAdmin`
- Only shown when user is not yet an admin (bootstrapping flow)
- Or: ENV-based auto-provisioning (no UI needed, handled by backend on startup)

### 6.5.2 Role Management (Admin Panel)

- Add "Role Management" tab to `/admin`
- Search user by DID or handle
- View current role + membership details
- Promote/demote buttons → `promoteRole` / `demoteRole` endpoints
- Audit log viewer — recent admin actions

### 6.5.3 Email Verification UI

- Add email verification flow to `/login` or `/dashboard`
- After OAuth login, prompt for institution email if no membership exists
- OTP input form → `verifyEmail` (step 1: request, step 2: verify)
- Show membership status on profile page

### Milestone 6.5 Checklist

- [ ] Admin provisioning form (or ENV-based bootstrapping)
- [ ] Role management UI in admin panel (promote/demote)
- [ ] Email verification flow (OTP request + verify)
- [ ] Membership status on profile/dashboard
- [ ] Audit log viewer (admin panel)

---

## Milestone 7: Search + Archives

> **Goal:** Unified search across all content. Archive browsing.
> **Depends on:** Milestones 1 + 4 (content exists to search).
> **Deliverable:** A student can search across notes, courses, brain nodes, and browse sealed archives.

### 7.1 Search Page (`/search`)

- Unified search bar (also accessible from header)
- Mode-aware: search scope changes with mode toggle
  - `academic` → `searchNotes` + `searchCourses`
  - `brain` → `searchBrainNodes`
  - `all` → all of the above, tabbed results
- Result cards: reuse `<CourseCard />`, `<NoteCard />`, `<BrainNodeCard />`
- Pagination

### 7.2 Archive Browser (`/archive`)

- `useGlobalArchiveFeed()` hook → `getGlobalArchiveFeed`
- `<ArchiveCard />` — course title, semester, session count, note count, sealed date, IA link
- Search within archives → `searchArchive`
- Click → archive detail (course + semester metadata, download links)

### Milestone 7 Checklist

- [x] Unified search page with mode-aware scoping
- [x] Tabbed results (courses, notes, brain nodes)
- [x] Header search bar integration
- [x] Archive browser with feed
- [x] `<ArchiveCard />` component
- [x] Archive search

---

## Milestone 8: Polish + Deploy

> **Goal:** Production-ready static build on Cloudflare Pages.
> **Depends on:** All previous milestones.
> **Deliverable:** `changala.app` is live.

### 8.1 Performance

- Route-based code splitting (`React.lazy` + `Suspense` for all page components)
- KaTeX CSS loaded only on pages that render LaTeX
- Image optimization (SVG logo, minimal assets)
- React Query cache tuning (stale times, garbage collection)

### 8.2 Error Handling

- Global error boundary
- Per-route error boundaries with retry
- XRPC error display (toast notifications for mutations, inline for queries)
- Offline detection + "no connection" banner

### 8.3 Accessibility

- Keyboard navigation for all interactive elements
- ARIA labels on mode toggle, notifications, graph
- Focus management on route changes
- Screen reader support for keyword histogram (data table fallback)

### 8.4 SEO + Meta

- `<title>` and `<meta>` tags per route (via `react-helmet-async` or `document.title`)
- Open Graph tags for shared links (course pages, brain nodes)
- `robots.txt` and `sitemap.xml` (static, for public pages)

### 8.5 Cloudflare Pages Deploy

- `wrangler.toml` or CF Pages dashboard config:
  - Build command: `cd web && npm run build`
  - Build output: `web/dist`
  - Environment variables: `VITE_RING_URL`, `VITE_GLOBALVIEW_URL`
- SPA fallback: `_redirects` file (`/* /index.html 200`) or `_routes.json`
- CORS: Ring's `atrg.toml` allows `https://changala.app` origin
- Preview deploys on PRs (CF Pages built-in)

### 8.6 CI Pipeline

- GitHub Actions workflow (`.github/workflows/web.yml`):
  - `npm ci` → `npm run lint` → `npm run build`
  - Triggered on changes to `web/**`
  - Deploy to CF Pages on merge to `main`

### Milestone 8 Checklist

- [x] Code splitting on all routes
- [ ] Error boundaries (global + per-route)
- [ ] Toast notification system for mutations
- [ ] Accessibility audit pass
- [ ] SEO meta tags
- [x] `_redirects` file for SPA routing
- [ ] Cloudflare Pages deployment config (`wrangler.toml`)
- [ ] GitHub Actions CI for `web/`
- [x] Environment variable documentation
- [x] Production build < 500KB gzipped (excluding KaTeX CSS)

---

## Dependency Map

```
M0 (Foundation)
 ├── M1 (Course Feed)
 │    ├── M2 (Live Session + Keywords)
 │    └── M7 (Search + Archives)
 ├── M3 (Auth + Votes + Identity)
 │    ├── M4 (Brain Layer)
 │    │    └── M5 (Graph Visualization)
 │    └── M6 (Collective Notes + Admin)
 │         └── M6.5 (RBAC + Admin Provisioning)
 └── M8 (Polish + Deploy) ← depends on all above
```

Milestones 1-2 and 3-4 can be worked on in parallel by different contributors once M0 is done.

---

## Directory Structure (Target)

```
web/
├── public/
│   ├── client-metadata.json    # AT Protocol OAuth metadata
│   ├── _redirects              # CF Pages SPA routing
│   └── favicon.svg
├── scripts/
│   └── lexicon-to-ts.ts        # Codegen: lexicon JSON → TypeScript types
├── src/
│   ├── generated/
│   │   └── types.ts            # Auto-generated from lexicons
│   ├── lib/
│   │   ├── xrpc.ts             # XRPC client (xrpcGet, xrpcPost)
│   │   └── auth.ts             # Token storage, header injection
│   ├── hooks/
│   │   ├── useXrpc.ts          # Generic query/mutation hooks
│   │   ├── useCourses.ts       # Course-related hooks
│   │   ├── useSessions.ts      # Session-related hooks
│   │   ├── useNotes.ts         # Note-related hooks
│   │   ├── useBrain.ts         # Brain node hooks
│   │   ├── useGraph.ts         # Graph hooks
│   │   ├── useAuth.ts          # Auth context hook
│   │   └── useNotifications.ts # Notification hooks
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Header.tsx
│   │   │   ├── ModeToggle.tsx
│   │   │   └── Layout.tsx
│   │   ├── content/
│   │   │   ├── ContentRenderer.tsx
│   │   │   ├── LatexRenderer.tsx
│   │   │   └── MarkdownRenderer.tsx
│   │   ├── academic/
│   │   │   ├── CourseCard.tsx
│   │   │   ├── SessionCard.tsx
│   │   │   ├── NoteCard.tsx
│   │   │   ├── KeywordHistogram.tsx
│   │   │   ├── KeywordInput.tsx
│   │   │   └── CollectiveNote.tsx
│   │   ├── brain/
│   │   │   ├── BrainNodeCard.tsx
│   │   │   ├── BacklinkList.tsx
│   │   │   ├── GraphView.tsx
│   │   │   └── MiniGraph.tsx
│   │   ├── common/
│   │   │   ├── VoteButton.tsx
│   │   │   ├── AuthorName.tsx
│   │   │   ├── NotificationBell.tsx
│   │   │   ├── ArchiveCard.tsx
│   │   │   └── SearchBar.tsx
│   │   └── admin/
│   │       ├── CourseForm.tsx
│   │       ├── ModerationPanel.tsx
│   │       └── ArchiveControls.tsx
│   ├── pages/
│   │   ├── LandingPage.tsx
│   │   ├── CourseCatalog.tsx
│   │   ├── CourseDetail.tsx
│   │   ├── SessionDetail.tsx
│   │   ├── LiveSession.tsx
│   │   ├── NoteViewer.tsx
│   │   ├── NoteEditor.tsx
│   │   ├── BrainFeed.tsx
│   │   ├── BrainNodeDetail.tsx
│   │   ├── BrainNodeEditor.tsx
│   │   ├── GraphExplorer.tsx
│   │   ├── ArchiveBrowser.tsx
│   │   ├── SearchPage.tsx
│   │   ├── ProfilePage.tsx
│   │   ├── LoginPage.tsx
│   │   ├── Dashboard.tsx
│   │   └── AdminPanel.tsx
│   ├── context/
│   │   ├── ModeContext.tsx
│   │   └── AuthContext.tsx
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── index.html
├── package.json
├── tsconfig.json
├── tsconfig.app.json
├── tsconfig.node.json
├── vite.config.ts
├── eslint.config.js
└── ROADMAP.md                  # ← you are here
```

---

## Environment Variables

| Variable | Example | Description |
|---|---|---|
| `VITE_RING_URL` | `https://nitc.changala.app` | Ring server base URL |
| `VITE_GLOBALVIEW_URL` | `https://changala.app` | Global View base URL (same as Ring in MVP) |
| `VITE_OAUTH_CLIENT_ID` | `https://changala.app/client-metadata.json` | AT Protocol OAuth client ID |

---

## Ship Order

The UIUX.md says it best: *"Ship the course feed first, then the keyword histogram, then the graph view."*

1. **M0** — Foundation (routing, XRPC, auth skeleton, layout)
2. **M1** — Course feed (prove the academic read loop)
3. **M2** — Live keywords (prove the real-time interaction)
4. **M3** — Auth + votes (prove authenticated writes)
5. **M4** — Brain layer (prove the Zettelkasten)
6. **M5** — Graph view (the demo that sells it)
7. **M6** — Collective notes + admin (class rep workflows)
8. **M6.5** — RBAC + admin provisioning (role management + email verification)
9. **M7** — Search + archives (discovery + preservation)
10. **M8** — Polish + deploy (ship to `changala.app`)

---

*Each milestone is independently demoable. Don't skip M0 — the foundation determines everything.*
