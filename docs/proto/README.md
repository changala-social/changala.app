# Proto Files — Reference Documentation Only

These `.proto` files are **not compiled** and are **not used at build time**. They serve as human-readable interface documentation for the Changala API surface.

The actual API is implemented via AT Protocol lexicon JSON files in `lexicons/` which are processed by `atrg generate` to produce typed Rust code and Axum XRPC route stubs.

## Why keep them?

Proto IDL is a clean, language-neutral way to describe service interfaces. These files document:
- The full RPC surface of the Ring and Global View services
- Request/response message shapes
- Shared enums and type definitions

They were the original design artifacts and remain useful as a cross-reference.

## Corresponding lexicons

| Proto service | Lexicon namespace |
|---|---|
| `IdentityService` | `app.changala.ring.verifyEmail`, `getMemberships`, `getRole` |
| `CourseService` | `app.changala.ring.createCourse`, `getCourse`, `listCourses`, etc. |
| `SessionService` | `app.changala.ring.createSession`, `openSession`, etc. |
| `NoteService` | `app.changala.ring.createNote`, `addKeyword`, etc. |
| `VoteService` | `app.changala.ring.registerVote` |
| `LabelService` | `app.changala.ring.applyLabel`, `retractLabel` |
| `ModerationService` | `app.changala.ring.banDid`, `liftBan`, etc. |
| `ArchiveService` | `app.changala.ring.initiateArchive`, `sealArchive`, etc. |
| `BrainService` | `app.changala.ring.createNode`, `createLink`, etc. |
| `FeedService` | `app.changala.globalview.getNotes`, `getCourseFeed`, etc. |
| `SearchService` | `app.changala.globalview.searchNotes`, `searchCourses`, etc. |
| `HistogramService` | `app.changala.globalview.getKeywordHistogram` |
| `GraphService` | `app.changala.globalview.getNodeGraph`, `getBacklinks`, etc. |
| `NotificationService` | `app.changala.globalview.getNotifications`, etc. |
