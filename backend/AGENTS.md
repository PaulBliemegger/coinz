# Backend guidelines

Read the root `AGENTS.md` first. Product behavior lives in `docs/specs/`; these
instructions cover Rust implementation choices.

## Modules and boundaries

- Keep one crate. Group substantial new areas by feature (ledger, imports,
  planning), with shared database startup, authentication, and error handling.
- The current `ledger.rs` and `auth.rs` are feature modules. Split a growing module
  into a directory when distinct responsibilities make navigation or testing hard.
  Add handlers, application operations, or query files only as needed.
- HTTP handlers extract requests, resolve the user, invoke operations, and map
  responses. Parsing/calculations should be testable without HTTP or SQLite.
- Keep SQL near the feature that uses it. `db.rs` owns connections and migration
  startup, not every query. Do not introduce repository traits or generic services
  merely to wrap SQLx.

## Rust and persistence

- Prefer concrete structs/enums, explicit validation, and owned values at request
  boundaries. Borrow within operations where it makes the code clearer.
- Use `Result` and `?` for expected failures. No `unwrap`, `expect`, or panics on
  user-controlled input or ordinary runtime failures; they are fine in tests with
  useful assertions. Comments explain a non-obvious reason or invariant.
- Keep blocking password hashing off async executor threads. Use maintained
  cryptographic/session libraries and their supported APIs.
- Validate money/date/text inputs before persistence. Enforce ownership in SQL,
  including related records; never accept an owner from the request body.
- Use parameterized SQL. Multi-step financial mutations and their invariant checks
  belong in one transaction; serialize SQLite writes when validation depends on
  the current balance. Roll back the whole operation on failure.
- Add versioned migrations with schema changes. Before v1.0 a documented reset can
  be appropriate, but never reset a database implicitly during startup.
- Return safe, actionable client errors. Log diagnostic causes without passwords,
  cookies, session tokens, uploaded rows, or other private financial details.
- API money fields are decimal strings of integer cents; avoid JSON-number loss
  of precision. Use stable snake_case field names and explicit request/response
  types. See the manual-ledger spec for the implemented contract.

## Verification

Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked --
-D warnings`, and `cargo test --workspace --locked`. The full `make check` adds
frontend and browser verification.

- Unit-test calculation and parsing edge cases; use real temporary SQLite
  databases for persistence, rollback, and ownership behavior.
- Cover permissions and failure paths when adding private routes, not just a
  successful request. Add concurrency coverage where a write invariant needs it.
- Reuse test helpers for setup, not for hiding the assertion or expected result.
- Test observable behavior and meaningful invariants; avoid mocking SQLx or
  snapshotting incidental implementation details.
