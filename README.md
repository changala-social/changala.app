# Changala

> A federated, open-protocol social learning platform and student knowledge graph, built on the AT Protocol.

Changala is built with [at-rust-go (atrg)](https://github.com/tellmeY18/at-rust-go) — a batteries-included AT Protocol backend framework for Rust.

## Quick Start

```bash
# Install the atrg CLI
cargo install atrg-cli

# Generate typed Rust code from lexicon definitions
atrg generate lexicons/

# Run the dev server
atrg dev
```

## Project Structure

```
changala/
├── Cargo.toml              # Dependencies
├── atrg.toml               # App config (OAuth, Jetstream, DB)
├── rust-toolchain.toml     # Pinned to stable Rust
│
├── lexicons/               # AT Protocol lexicon JSON files (source of truth)
│   └── app/changala/
│       ├── defs.json           # Shared types (RingRef, enums)
│       ├── membership.json     # Institution affiliation record
│       ├── course.json         # Course record
│       ├── session.json        # Class session record
│       ├── keyword.json        # Keyword submission record
│       ├── note.json           # Note metadata record
│       ├── vote.json           # Vote record
│       ├── label.json          # Quality label record
│       ├── archive.json        # Sealed archive record
│       ├── collectivenote/
│       │   └── proposal.json   # Edit proposal record
│       ├── brain/
│       │   ├── node.json       # Brain node record
│       │   └── link.json       # Brain link record
│       ├── ring/               # Ring XRPC procedures & queries
│       │   ├── verifyEmail.json
│       │   ├── createCourse.json
│       │   └── ...
│       └── globalview/         # Global View XRPC queries
│           ├── getNotes.json
│           ├── searchNotes.json
│           └── ...
│
├── src/
│   ├── main.rs             # Entry point
│   ├── routes.rs           # Route wiring
│   ├── generated/          # ← OUTPUT of `atrg generate` (gitignored)
│   │   ├── mod.rs
│   │   ├── types.rs        # Serde-derived structs
│   │   └── routes.rs       # XRPC handler stubs + xrpc_routes()
│   └── handlers/           # Your business logic implementations
│       ├── mod.rs
│       ├── identity.rs
│       ├── session.rs
│       ├── note.rs
│       ├── brain.rs
│       └── ...
│
├── migrations/             # SQLite migrations
│   └── .gitkeep
│
└── docs/
    └── proto/              # Reference documentation (NOT compiled)
        ├── README.md
        └── *.proto
```

## Architecture

Changala has two logical layers:

- **Academic Layer** — Courses, sessions, class notes, keyword histograms, archives
- **Brain Layer** — Personal knowledge nodes (Zettelkasten), backlinks, tags, graph traversal

And three runtime components:

- **Ring** — Content server for heavy blobs (LaTeX, PDFs, markdown). One per institution.
- **Global View** — Read-layer aggregator subscribing to the ATProto firehose. Materialises feeds, search, histograms, and the brain graph index.
- **API Gateway** — Axum-based XRPC handlers generated from lexicon definitions.

## Development

```bash
# Generate code from lexicons (run after any lexicon change)
atrg generate lexicons/

# Run migrations
atrg migrate

# Start dev server with hot reload
atrg dev

# Print all registered routes
atrg routes
```

## License

LGPL-3.0-only
