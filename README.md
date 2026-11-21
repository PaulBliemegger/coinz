# coinz

A personal finance notebook: clear spending insights, warm colors, and occasional
dry humor. Rust/Axum serves a static SvelteKit interface and stores data in SQLite.

## Current state

Implemented: local user creation, login/logout, isolated user data, manual income
and expenses without setup, optional source metadata, editing/deleting entries,
month/source filters, and recorded monthly income/spending/net totals.

The core is Transactions, Insights, and Planning. Richer insights, labels, CSVs,
and planning remain future work; no empty placeholder screens are added. Bank
balance tracking and reconciliation are not required. A balance-based What-if
calculator may follow when useful. See [product scope](docs/product.md) and the
[planning direction](docs/specs/planning.md).

Migration 0003 preserves old account names/IDs as transaction sources and makes the
entry source optional. Old opening figures and snapshots remain stored but have
no active screen/API and do not affect reports. Back up private data before
reviewing/deploying changes; the agent does not inspect your running database.

Rust tests/lint, frontend checks/tests, and native production builds pass. Browser
journeys remain unverified: the sandbox denies the synthetic server's local socket.
An earlier Docker image built successfully; this revision has not been deployed or
verified against a running container by the agent. This is an early development
version.

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

Open [http://localhost:5173](http://localhost:5173). The frontend proxies `/api` to
Rust on `127.0.0.1:3000`; `make dev` configures the matching frontend origin.
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

- [Overall guidelines](AGENTS.md), [backend guidelines](backend/AGENTS.md), and
  [frontend guidelines](frontend/AGENTS.md).
- [Architecture and repo structure](docs/architecture.md).
- [Product scope](docs/product.md) and [specification index](docs/specs/README.md).

Update the relevant specification and acceptance tests in the same change as
behavior. Keep setup commands here and executable commands in the Makefile;
feature documents link to them instead of maintaining their own setup copies.
