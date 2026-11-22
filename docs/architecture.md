# Architecture and repository structure

This document owns runtime structure and technical decisions. See the
[specification index](specs/README.md) for feature status and behavior, and the
[README](../README.md#verification-status) for current verification limits.

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
    ledger.rs             Optional sources, entry writes and legacy monthly reads
    reports.rs            Paginated entry queries and dashboard read snapshots
    imports/              CSV parsing, preview, matching, commit and batch history
    domain.rs             Pure parsing/calculation rules
    db.rs                 SQLite setup and migrations
    error.rs              Shared safe API errors
  migrations/             Versioned SQL
  tests/ledger.rs         Temporary SQLite and API integration tests
frontend/
  AGENTS.md               Svelte/TypeScript conventions
  src/routes/             Home, Entries, Imports routes and shared page styles
  src/lib/App.svelte      Session shell, navigation, shared entry/import actions
  src/lib/features/       Dashboard, Entries, import review and batch removal
  src/lib/components/     Reused visual/interaction primitives
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
  feature-workflow.md     Small feature chats, reviews and temporary handovers
Dockerfile / compose.yaml Deployment definition
Makefile                  Named, repeatable development/check commands
```

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
  idempotent. Original file/record and source-scoped bank evidence live separately
  from mutable transactions (migration 0004); raw uploads are not persisted.
  Deletion clears linked evidence and permits reimport (migration 0006).
  Batch counts/member roles and transaction revisions (migration 0007) support
  removal with a review fingerprint rechecked under the write lock. Existing
  `added` versus `known`/`matched` roles express one creating batch without an
  additional ownership table. See [import history](specs/import-history.md) for
  removal rules and the [CSV spec](specs/csv-import.md) for matching semantics.
- Field-specific PATCH operations preserve unrelated entry edits. Migration 0005
  stores explicit entry origin independently of editable source metadata.
  Migration 0003 preserves former accounts as sources and retires balance tables;
  the [ledger spec](specs/manual-ledger.md#preserved-legacy-data) owns those guarantees.
- The session shell coordinates shared entry/import actions and refreshes active
  feature reads after mutations. Feature components ignore superseded responses.
  Reporting uses checked integer totals in Rust and one read transaction per
  response, avoiding SQLite integer SUM overflow. It loads matching amounts,
  not all entry metadata; streaming aggregation can follow if volume warrants it.
- Appearance initializes before paint through CSS theme tokens. Local view
  preferences contain no financial or authentication data; see
  [appearance behavior](specs/manual-ledger.md#appearance-and-user-menu).
  Entries and selectable import rows share components and Svelte actions under
  `src/lib/components/selection/`. Separate fade and rotating-background layers
  preserve 3D faces, cancel interrupted effects, and clean up listeners. No
  animation package or backend selection state is needed. The dashboard's
  month-only picker uses native buttons and a year input, avoiding a day-calendar
  dependency; [reporting](specs/reporting.md) defines navigation and focus behavior.
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
