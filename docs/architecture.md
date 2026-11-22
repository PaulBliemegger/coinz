# Architecture and repository structure

Status: the manual ledger, example CSV importer, inline editing, bulk deletion,
and view preferences are implemented. Browser/visual and container runtime
verification remain pending. Product behavior lives in [specs](specs/README.md);
commands and operations live in the [README](../README.md).

## Runtime

One Rust/Axum process serves `/api` and the built SvelteKit 3 static interface.
SQLite uses SQLx, versioned migrations, foreign keys, WAL, and a busy timeout.
There is one Rust crate and one frontend package. Docker uses separate build
stages; only the binary, static assets, and runtime system dependencies ship.

The application tracks `Cargo.lock`; Rust checks and production builds use
`--locked` to keep native and container builds on the reviewed dependency resolution.

During development, Vite serves the UI and proxies `/api` to Rust. Deployment uses
one origin. Rust owns authorization, validation, persistence, and financial
calculations. The browser owns interaction and exact presentation of cents
strings. Credentials/CSRF handling stay at the HTTP boundary.

Rust is an intentional learning choice, not a response to high throughput needs.
SQLite avoids a separate service for personal use. Responsive web covers laptop
and phone use; native packaging and offline synchronization are deferred.

## Current layout

```text
AGENTS.md                 Overall conventions and working agreement
backend/
  AGENTS.md               Rust-specific conventions
  src/
    main.rs               CLI, configuration, server lifecycle
    lib.rs                Application/router assembly
    auth.rs               Identity, sessions, CSRF
    ledger.rs             Optional sources, entries, summaries and their SQL
    imports/              CSV parsing, preview, matching and atomic commit
    domain.rs             Pure parsing/calculation rules
    db.rs                 SQLite setup and migrations
    error.rs              Shared safe API errors
  migrations/             Versioned SQL
  tests/ledger.rs         Real database and API integration tests
frontend/
  AGENTS.md               Svelte/TypeScript conventions
  src/routes/             Screen composition and page styles
  src/lib/features/       Import review and ledger list/inline entry interactions
  src/lib/api.ts          HTTP types/client and session token boundary
  src/lib/money.ts        Exact amount/date display helpers
  tests/                  Focused frontend logic tests
  e2e/                    Browser journeys and isolated test server
scripts/                  Small development helpers
.githooks/                Explicitly installed Git hooks
docs/
  product.md              Goals, scope, sequencing, open product choices
  architecture.md         Runtime, boundaries, structure and major tradeoffs
  specs/                  Detailed behavior and acceptance examples by feature
  agent-testing.md        Sandbox-specific test guidance
Dockerfile / compose.yaml Deployment definition
Makefile                  Named, repeatable development/check commands
```

## Product boundary

Transactions, period insights, and planning are the core. Optional import/source
metadata supports provenance, duplicate matching, and future transfer recognition.
No account bookkeeping or balance maintenance is required. Planning is not yet
implemented; its future monthly scenarios can work without a starting bank balance.
A balance-based What-if calculator is an optional later capability, not a runtime
or schema dependency of expense tracking.

## How this grows

Use a light hybrid: feature modules for application behavior and a small shared
infrastructure boundary. Do not organize the whole app into global controllers,
services, repositories, and DTO directories, which scatter each feature.

Keep the present flat Rust modules while they remain understandable. As a feature
grows, a module such as `ledger.rs` can become `ledger/` with only the submodules
it needs: HTTP handlers, operations, queries, or types. Imports should have their
own source adapters and normalization logic rather than being added to ledger
handlers. Pure financial rules remain independently testable. Cross-feature
operations should call explicit operations, not another feature's HTTP handler.

On the frontend, routes compose feature components. Extract source/transaction
forms and ledger panels when changing those independently. Once justified, keep
related UI under `src/lib/features/ledger/` or `features/imports/`; shared UI
primitives belong under `src/lib/components/` only when reused. Avoid empty
scaffolding directories, generic utility collections, and required layers.

## Guardrails

- Ownership comes from the server session and is enforced in database operations,
  including related source/entry IDs.
- Exact financial rules, dates, source provenance, and incomplete-history semantics
  follow the feature/product specs. Schema constraints and tests reinforce them.
- Ledger mutations are atomic and serialized with `BEGIN IMMEDIATE`. Transactions
  own a user independently of the optional source. The source join is a left join
  so manual entries remain in reports. Composite foreign keys enforce source
  ownership when a source is supplied.
- Imports reparse a bounded CSV for preview/commit, keeping preview stateless.
  Commit revalidates under `BEGIN IMMEDIATE` so repeated/concurrent submits are
  idempotent. Migration 0004 stores file/record and source-scoped bank evidence
  separately from mutable transactions, preserving edits. Migration 0006 releases deleted entries for reimport,
  removes old accepted deletion markers, and deletes linked evidence before entry deletion.
  The raw uploaded file is not persisted. The import dialog is a separate frontend
  feature component; the ledger page coordinates refresh and filters.
- Migration 0005 adds explicit entry origin without guessing legacy history.
  Field-specific PATCH operations preserve unrelated edits; bulk deletion validates
  every owned ID in one write transaction. Import selection is revalidated at
  commit. Selection visibility stores a local boolean; appearance stores a validated enum,
  initialized before paint and applied through CSS theme tokens. Neither contains
  financial data. Appearance defaults to OS-driven System mode.
- Source metadata is not a balance ledger. Migration 0003 preserves old account
  identities as sources and retains retired balance tables without exposing an API.
  The optional future calculator must not depend on legacy observations.
- Password hashing uses Argon2id; `tower-sessions` and its SQLx SQLite store manage
  opaque sessions. No custom crypto or session-token implementation.
- Migrations run on connection/startup. Expired session records are cleaned at app
  assembly; session loading enforces expiry. Additional cleanup can follow usage.
- Tests use synthetic data and temporary databases. Full checks run through
  `make check`; CI should reuse those commands when added.

## Decision records

Keep the rationale above while it is short. Introduce `docs/decisions/` only when
an important tradeoff needs a longer durable explanation (for example changing
storage or adding synchronization). Record context, choice, consequences, and
supersession; do not duplicate feature specifications or add records for routine
implementation choices.
