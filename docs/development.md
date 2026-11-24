# Development

This guide owns local setup, checks and disposable database resets. See
[deployment](deployment.md) for container images/runtime configuration and
[privacy](privacy.md) for the synthetic-data boundary.

## Prerequisites and startup

Requirements: Bash, Make, a C compiler/build tools for SQLite, rustup, Node
**24.22.3**, and pnpm **11.25.0**. The Rust version is pinned in
[rust-toolchain.toml](../rust-toolchain.toml); rustup installs/selects it when Cargo
runs in this checkout. `.node-version` records the tested Node version.

Install rustup using the [official Rust instructions](https://www.rust-lang.org/tools/install)
and Node using your usual version manager or [Node downloads](https://nodejs.org/en/download).
With Node installed, install the pinned package manager:

```sh
npm install --global pnpm@11.25.0
make setup
cargo run --locked -- create-user wabo
make dev
```

Open [http://localhost:5173](http://localhost:5173). Use this exact host: `127.0.0.1:5173`
is a different origin and `make dev` rejects its writes. An origin mismatch names
the configured app address; a CSRF/session mismatch asks you to refresh.
The frontend proxies `/api` to Rust on `127.0.0.1:3000`; `make dev` configures the matching frontend origin.
Ctrl-C stops the development launcher. You can instead run `make backend` and
`make frontend` in separate terminals.

The initial Rust build downloads and compiles dependencies and can take several
minutes. Later builds reuse `target/`. No external database server is needed.

To opt into the included pre-commit hook:

```sh
make hooks
```

It runs `make quickcheck` on the working tree. Hook installation is optional and
changes your local Git configuration.

`Cargo.lock` is tracked for this application. Rust checks and production builds use
`--locked` so they use the reviewed dependency resolution. Include lockfile changes
when intentionally updating Rust dependencies.

## Planned maintenance

The temporary [code cleanup plan](code-cleanup-plan.md) describes the proposed
behavior-preserving slices from the repository review. It is separate from
[Pi deployment preparation](deployment.md#first-pi-trial); remove the plan and this
link when the work is complete or superseded.

## Tests and checks

```sh
make format          # Apply Rust and frontend formatting
make quickcheck      # Formatting checks and frontend lint
make browser-install # One-time Chromium download for Playwright
make check           # Lint, tests, type checks, builds, desktop/phone journeys
```

Browser installation may report missing Linux system libraries; install the
packages requested by Playwright for your distribution. If you already have a
compatible Chromium browser, select its executable explicitly:

```sh
PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH=/absolute/path/to/chromium make check
```

Individual checks:

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm --dir frontend check
pnpm --dir frontend test
```

Run browser tests locally from your terminal before pushing. These targets build
the same production artifacts as CI and start the existing isolated test server:

```sh
make browser-test
make browser-test PLAYWRIGHT_ARGS='--project=phone --grep="CSV subset"'
make browser-debug PLAYWRIGHT_ARGS='--project=phone --grep="CSV subset"'
```

`browser-debug` opens the browser and Playwright Inspector so you can step through
the failing action and inspect its locator. Start with a single failing journey,
fix and rerun it locally, then run `make check` before pushing. CI verifies the
result. No Git commit or GitHub run is needed for each debugging attempt.
See [Playwright's debugging options](https://playwright.dev/docs/test-cli#common-options).

Each invocation creates a fresh synthetic database and removes it when the test
server exits. Prefer restarting the command between debugging attempts: the current
journeys share their project user's state within a server lifetime, so repeated
execution in a long-running UI/watch session can collide with earlier fixture
names. App changes require rebuilding the artifacts; the Make targets do that.
For test-only changes, rerun `pnpm --dir frontend exec playwright test` with the
same project/title arguments to reuse the existing builds.

Keep the full suite at one worker: several files share and clean up the same
project user's entries. Raising the global worker count requires isolated test
data first. The settings portability desktop/phone journeys use separate accounts
and can run together with
`make browser-test PLAYWRIGHT_ARGS='e2e/settings-portability.spec.ts --workers=2'`.

Playwright uses `127.0.0.1:4100`, synthetic credentials, and a temporary database.
It does not use your development or Compose database. Close anything already
using port 4100. Failure traces/screenshots go under `frontend/test-results/`.

A restricted agent sandbox must permit local sockets/listening and browser
processes for these tests. A startup timeout alone does not prove an app bug.
See [agent testing](agent-testing.md). Tests run in your own terminal
are independent of the agent's sandbox policy.

## Development schema baseline and reset

`backend/migrations/0001_initial.sql` is the clean pre-v0.1 schema baseline.
The earlier 14-migration development history is no longer supported. New databases
start directly with the active schema; retired opening-balance/snapshot tables
are omitted. Future schema changes use new numbered migrations.

An existing database from the earlier history will fail migration validation.
Startup never deletes data or rewrites migration checksums. For disposable local
development data using the default `data` directory, stop `make dev` with Ctrl-C,
then run from the repository root:

```sh
rm -f -- data/coinz.sqlite3 data/coinz.sqlite3-wal data/coinz.sqlite3-shm
cargo run --locked -- create-user wabo
make dev
```

This deliberately removes all local users, sessions and financial/setup data;
choose your username when recreating the account. If you set `COINZ_DATA_DIR`,
use that directory instead of `data`. No export or restoration is needed for
disposable fixtures; import the synthetic demo again if desired.

When keeping data across a future deliberate development reset, use the
[settings JSON and entry CSV workflow](specs/data-portability.md).
A checksum error on a database already using this baseline can also indicate
modified migration bytes: keep SQL files at LF and rebuild before diagnosing it.
Do not bypass checksum validation or delete migration metadata.

## Verification status

Snapshot, 2026-11-24: after the clean schema reset, the owner confirmed the local
app works and the tests passed. Earlier owner-run checks also covered CSV and
settings JSON restoration. The agent-run baseline checks passed formatting/lint,
Clippy, 57 Rust tests, frontend type checks, 9 frontend unit-test files and both
production builds. Agent-run Playwright reached server startup but the sandbox
denied sockets; the owner subsequently reported a passing run. Owner-run results
are reported evidence, separate from checks executed by the agent.

The shared synthetic UI was reviewed at desktop/phone widths during portability
work. Native and browser checks do not establish Pi runtime performance or prove
that the latest container image has been deployed. The first Pi trial remains to
be performed; see [deployment](deployment.md#first-pi-trial).

Use the pinned toolchain versions above. Some prior agent checks used Node 26 and
reported an engine warning; that does not change the supported Node version.
Future feature work should report its executed checks and limitations when handed
off. Update this snapshot at a meaningful verification boundary rather than
copying session status into every feature spec.
