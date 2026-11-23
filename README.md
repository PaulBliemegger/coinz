# coinz

A personal finance notebook: clear spending insights, warm colors, and occasional
dry humor. Rust/Axum serves a static SvelteKit interface and stores data in SQLite.

## Current state

Implemented: local users and login, manual income/expenses, optional sources,
a monthly/yearly spending dashboard, paginated Entries with filters and inline
editing, reusable CSV profiles with guided header mapping and editable import
review, duplicate protection and protected batch removal.
Create or edit reusable layouts in **Imports → Import profiles**, without
uploading a file. Changes affect future imports only.
Appearance follows the system by default, with Light/Dark overrides.

Try **Import CSV** on Home, Entries, or Imports using [demo.csv](examples/imports/demo.csv).
Create a fresh source and profile; map Date to `date`, Amount to `amount`, and
Description to `description`. Choose `external_id` for the optional ID mapping.
Keep the default reading settings, name/save the profile, review the 16 synthetic
entries, then import. Reuse that profile for subsequent example files. Choose
November 2026 in the centered dashboard month picker to see €2,400.00 income and
€1,063.79 spending, or use View all to see both imported months.
Importing the same file again adds nothing;
[demo-overlap.csv](examples/imports/demo-overlap.csv) adds just a €16 cinema expense.
Use a synthetic development runtime for this walkthrough.

The [CSV profile presets](docs/specs/import-profiles.md) support UTF-8 header CSVs,
comma/semicolon delimiters, three explicit date formats and exact signed EUR
amounts. Bank-specific adapters, transfer/refund classification, labels, YAML
profiles and planning remain deferred. See [product scope](docs/product.md) for
sequencing and the
[specification index](docs/specs/README.md) for detailed behavior and feature status.
Bank balance tracking and reconciliation are outside the core scope.

### Verification status

Verification snapshot (2026-11-23): `make check` passed formatting, lint, all 29
Rust tests, frontend type/unit checks, and native production builds. It stopped at Playwright server
startup because the agent sandbox denies local sockets; browser assertions and
visual review remain unverified. See [agent test permissions](docs/agent-testing.md).
The frontend checks ran with Node 26.13.1 and reported an engine warning;
the supported project version remains Node 24.22.3.
An earlier Docker image built successfully; the current revision has not been
deployed or verified against a running container by the agent. This is an early
development version.

## Real data privacy

Before entering real finances, read [privacy and development isolation](docs/privacy.md).
Use a development VM with synthetic data and a separate private runtime. The
current agent permissions, Git ignore rules, and default Compose network do not
guarantee that real data is inaccessible or unable to leave the laptop. Do not
expose the private database, app, logs, browser profile, or host Docker daemon to
the development agent.

## Run with Docker Compose

Requirements: Docker Engine with the Compose and Buildx plugins. Check:

```sh
docker version
docker compose version
docker buildx version
```

If Compose is missing, follow Docker's [Linux Compose installation guide](https://docs.docker.com/compose/install/linux/).
For an Engine installation, use the [distribution-specific instructions](https://docs.docker.com/engine/install/),
including the Compose and Buildx packages. Temporary standalone tools used during
agent verification do not install those plugins on your host.

On Linux Mint/Ubuntu with the distribution's `docker.io` engine, the matching
packages are `docker-compose-v2` and `docker-buildx` (installed through apt).
The names `docker-compose-plugin` and `docker-buildx-plugin` belong to Docker's
own repository. Keep the engine and plugins on a consistent package source.
For Mint, Docker's Ubuntu repository uses `UBUNTU_CODENAME` (e.g. `noble`), not
Mint's `VERSION_CODENAME` (e.g. `zena`); a Debian repository with suite `zena`
will fail. The distribution packages do not require adding Docker's repository.

From the repository root:

```sh
docker compose build
docker compose run --rm coinz create-user wabo
docker compose up -d
```

The user-creation command prompts for a password and confirmation (12–128
characters). Choose your own username; there are no default credentials or public
registration. Open [http://localhost:3000](http://localhost:3000) and log in.
Use that exact hostname: the configured origin is `http://localhost:3000`.

```sh
docker compose ps
docker compose logs -f coinz
docker compose down
```

Compose binds the app to your laptop's loopback interface. SQLite lives in the
`coinz-data` named volume mounted at `/data`, independently of the container.
`docker compose down` preserves it. Adding `--volumes` removes its data.
Development without Docker uses a separate database under `data/`.

To update after source changes, run `docker compose up -d --build`. Migrations run
on startup. Before v1.0, schema compatibility across development changes is not
guaranteed; take a backup before updating data you want to keep.

## Local development

Requirements: Bash, Make, a C compiler/build tools for SQLite, rustup, Node
**24.22.3**, and pnpm **11.25.0**. The Rust version is pinned in
[rust-toolchain.toml](rust-toolchain.toml); rustup installs/selects it when Cargo
runs in this checkout. `.node-version` records the tested Node version.

Install rustup using the [official Rust instructions](https://www.rust-lang.org/tools/install)
and Node using your usual version manager or [Node downloads](https://nodejs.org/en/download).
With Node installed, install the pinned package manager:

```sh
npm install --global pnpm@11.25.0
make setup
cargo run -- create-user wabo
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
Temporary `/tmp/coinz-*` tools used during agent verification are not prerequisites
and are not a persistent installation on your laptop.

After initializing Git, enable the included pre-commit hook:

```sh
make hooks
```

It runs the fast checks on the working tree. No Git initialization, commit, remote,
or hook installation has been performed automatically.

`Cargo.lock` is tracked for this application. Rust checks and production builds use
`--locked` so they use the reviewed dependency resolution. Include lockfile changes
when intentionally updating Rust dependencies.

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

To run only browser tests, first build the artifacts they serve:

```sh
pnpm --dir frontend build
cargo build --release --locked
pnpm --dir frontend test:e2e
```

Playwright uses `127.0.0.1:4100`, synthetic credentials, and a temporary database.
It does not use your development or Compose database. Close anything already
using port 4100. Failure traces/screenshots go under `frontend/test-results/`.

A restricted agent sandbox must permit local sockets/listening and browser
processes for these tests. A startup timeout alone does not prove an app bug.
See [agent test permissions](docs/agent-testing.md). Tests run in your own terminal
are independent of the agent's sandbox policy.

## Configuration

Pass these environment variables directly to the process, or in Compose's
`environment` section. The backend does not automatically load `.env` files.

| Variable | Native default | Purpose |
| --- | --- | --- |
| `COINZ_DATA_DIR` | `data` | SQLite directory; Compose uses `/data` |
| `COINZ_BIND` | `127.0.0.1:3000` | Listener; container uses `0.0.0.0:3000` |
| `COINZ_ORIGIN` | `http://localhost:3000` | Exact browser origin for write requests |
| `COINZ_STATIC_DIR` | `frontend/build` | Built frontend; container uses `/app/frontend` |
| `COINZ_COOKIE_SECURE` | `false` | Set `true` when accessed through HTTPS |
| `RUST_LOG` | `coinz=info,tower_http=info` | Backend logging filter |

Passwords can also be supplied via stdin using `create-user USER --password-stdin`.
Use that with a password manager or secret source; avoid putting passwords in
command arguments or shell history.

## Later homelab deployment

Keep the same image and a persistent SQLite directory on the NAS's local disk.
Place an HTTPS reverse proxy in front of the app, set `COINZ_ORIGIN` to its actual
browser origin, and set `COINZ_COOKIE_SECURE=true`. Adjust Compose networking so
that proxy can reach the container; the supplied port mapping is laptop-only.
The origin and cookie settings must match how you access the app.

The homelab repo can own deployment overrides and image selection, while this
repo owns application code and its Dockerfile. A sibling checkout or a separately
built image is sufficient initially; a Git submodule is not required.

For a straightforward backup, stop all coinz processes that use the database and
copy the complete data directory/volume, including any SQLite WAL sidecars. Keep
the backup private. Restore into an empty data volume with ownership writable by
UID/GID `10001:10001`, then start the matching app version. Test restore before
relying on homelab data. Do not copy only a running SQLite main file.

## Working on the project

Each document has a specific responsibility; link to the authoritative rule
instead of copying it into another guide.

| Document | Owns |
| --- | --- |
| This README and the Makefile | Setup, commands, deployment, and current verification status |
| [Overall guidelines](AGENTS.md), [backend](backend/AGENTS.md), [frontend](frontend/AGENTS.md) | Coding and verification conventions |
| [Product scope](docs/product.md) | Goals, boundaries, and next slices |
| [Feature specifications](docs/specs/README.md) | Detailed behavior, API contracts, and acceptance evidence |
| [Architecture](docs/architecture.md) | Technical structure and material decisions |
| [Design philosophy](docs/design.md) | Visual and interaction principles |
| [Privacy](docs/privacy.md) | Private runtime and development isolation |
| [Feature workflow](docs/feature-workflow.md) | Feature chats, reviews, and temporary handovers |

Update the affected specification and acceptance tests alongside behavior.
Keep setup commands here and executable commands in the Makefile.
