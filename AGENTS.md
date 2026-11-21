# Working on coinz

coinz is a small personal finance web app. Read `docs/product.md` for scope and
`docs/architecture.md` for implementation decisions before changing behavior.
Read `docs/specs/README.md` and the relevant feature spec. Scoped backend/frontend
`AGENTS.md` files add language-specific rules without duplicating this file.

## Collaboration

- Make routine implementation decisions and finish the agreed work. Explain
  material tradeoffs. Ask short questions when the answer changes product scope.
- Work in small vertical slices with concrete acceptance examples. Maintain only
  one active slice; avoid speculative infrastructure or a large up-front backlog.
- UI changes should be responsive, accessible, and visually reviewed using sample
  data. Prefer simple flows and make financial assumptions visible.
- Do not create commits, remotes, or a branch workflow without user instruction.

## Private data boundary

- Never read personal databases, bank exports, backups, private runtime logs, or
  screenshots/traces of real finances. Never query the real app or attach to its
  container, data volume, or authenticated browser session.
- Work only with source/docs and wholly synthetic test fixtures in temporary
  databases. Scope searches explicitly; do not scan private data directories.
- Do not use Docker daemon access, escalated commands, or another tool surface
  to get around data isolation. See `docs/privacy.md`; these instructions are
  additional workflow rules, not a substitute for OS/VM enforcement.

## Architecture and conventions

- Prefer specifically named feature modules and small shared infrastructure.
  Follow the current/growing structure in `docs/architecture.md`; create folders
  when code needs them, not to complete a template.
- Use consistent names and explicit types at boundaries. Keep functions focused;
  extract independently understandable responsibilities instead of enforcing
  arbitrary file/function length limits. Avoid speculative abstractions.
- Format mechanically. Comments explain reasons or invariants rather than
  narrating syntax. Prefer clear control flow and actionable errors.

- Rust + Axum backend; Svelte + TypeScript through SvelteKit's static adapter;
  SQLite through SQLx. One deployed app container serves both API and frontend.
- Keep period calculations and import matching separate from HTTP handlers and UI.
  Use ordinary modules and concrete types. Add abstractions for a demonstrated
  need, not for a hypothetical future integration.
- Store EUR money as signed integer cents; never use floating point for financial
  calculations. Document and test any conversion or rounding at input boundaries.
- Treat transaction dates as calendar dates. Define report boundaries explicitly.
- Scope every private read and write to the authenticated user. Test isolation
  with two users, including imports, labels, budgets, and reports.
- Keep business rules on the backend. Never trust client ownership IDs or totals.
- Preserve import provenance and user edits. Never silently discard ambiguous
  duplicate candidates. See the product document for duplicate semantics.
- Keep real exports, credentials, and databases out of version control and logs.
  Use synthetic fixtures that exercise the actual format being supported.
- Use established libraries for password hashing and cookie sessions. Include
  CSRF protection for cookie-authenticated writes; do not invent authentication
  primitives.

## Verification

Run `make quickcheck` for formatting/frontend lint and `make check` for the full
suite, including Rust lint/tests, frontend type/unit checks, production builds,
and Playwright journeys. `make format` applies formatting. See the README for
prerequisites and scoped instructions for individual checks. Do not claim the
full suite passed if browser or other required checks could not run.

- Pre-commit checks should be quick. Run the full checks before handing off a
  completed implementation slice, and report any failures or unavailable checks.
- Prefer Rust unit tests for calculations and parsing, temporary SQLite database
  integration tests for persistence and ownership, and a few Playwright journeys.
- Cover repeat imports, overlapping exports, valid identical purchases, manual
  matching, transfers, refunds, label overlap, and forecast assumptions as their
  features are implemented. Do not implement all these features just to test them.
- Add regression coverage for meaningful bug fixes. Avoid tests that merely
  restate implementation details or inflate coverage numbers.

## Documentation and completion

- Update affected docs in the same change as behavior. Keep each rule in one
  authoritative place and link to it elsewhere.
- `README.md`: setup, commands, current runnable status.
- `docs/product.md`: goals, scope, sequencing, and unresolved product choices.
- `docs/specs/`: authoritative detailed feature behavior and acceptance evidence.
  Label implemented, proposed, and deferred behavior explicitly. Split documents
  at independently changing feature boundaries; update links instead of copying.
- `docs/architecture.md`: structure and material technical decisions, with reasons.
- Remove completed temporary plans. Do not maintain parallel copies of the roadmap
  or document every file or function.
- A completed slice has working behavior, relevant checks, current docs, and a
  concise account of what changed and any remaining limitations.
