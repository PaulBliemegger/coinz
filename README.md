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
Create a profile (Source is optional under import details); map Date to `date`, Amount to `amount`, and
Description to `description`. Choose `external_id` for the optional ID mapping.
Keep the default reading settings, name/save the profile, review the 106 synthetic
entries, then import. Reuse that profile for subsequent example files. Choose
November 2026 on Home or Insights to see €2,585.00 income and
€1,596.88 spending, or use View all to explore September–November.
Importing the same file again adds nothing;
[demo-overlap.csv](examples/imports/demo-overlap.csv) adds just a €16 cinema expense.
Use a synthetic development runtime for this walkthrough.
See the [demo workflow](examples/imports/README.md) for suggested local rules,
trip labels and checks. Use a fresh synthetic source when replacing the older demo;
this expanded fixture has new IDs and does not replace previously imported entries.

The [CSV profile presets](docs/specs/import-profiles.md) support UTF-8 header CSVs,
comma/semicolon delimiters, three explicit date formats and exact signed EUR
amounts. Categories and optional coloured labels are available through **Organize**,
manual entry, inline/bulk editing, import review, and Entries filters. Setup is skippable and
existing entries remain Uncategorized. See [classification](docs/specs/classification.md).
**Organize → Rules** creates local description rules for categories/labels.
Import review shows editable suggestions and category conflicts; manual entry
can accept suggestions or keep its own choices. Preview history explicitly before
applying a rule to existing entries. Category/label deletion previews affected
entry/rule counts. See [local rules](docs/specs/labeling-rules.md).
Bank-specific adapters, transfer/refund handling, YAML
profiles and advanced planning remain deferred. See [product scope](docs/product.md) for
sequencing and the
[specification index](docs/specs/README.md) for detailed behavior and feature status.
Bank balance tracking and reconciliation are outside the core scope.

Home's **Spending by category** panel breaks down the selected month's recorded
spending, including archived and Uncategorized entries. Click a bar to open its
spending entries. Entries groups **All / Expenses / Income**, searchable category
and label pickers, and removable active-filter chips together. Clear filters keeps
the selected dates and description search.
See [reporting](docs/specs/reporting.md#spending-by-category).
The **Insights** tab shows the full monthly report plus annual income
versus expenses and category spending charts, with monthly/average expense splits. Bars link to their entries; exact
figures are available in tables. Home's **Explore insights** link carries over
the selected month.

### Verification status

Verification snapshot (2026-11-24): `make check` passed formatting, lint, all 54
Rust tests, frontend type/unit checks, and native production builds. It stopped at Playwright server
startup because the agent sandbox denies local sockets. An initial GitHub run
executed all 42 desktop/phone browser journeys: 20 passed and 22 failed. Fixes
address dropdown label lookup, inline-edit selector ambiguity and an Insights
fixture assigning an archived category; a successful GitHub rerun and visual
review are still required. An offline Chromium launch was also unavailable.
See [agent test permissions](docs/agent-testing.md).
The frontend checks ran with Node 26.13.1 and reported an engine warning;
the supported project version remains Node 24.22.3.
An earlier Docker image built successfully; the current revision has not been
deployed or verified against a running container by the agent. This is an early
development version.

### Migration checksum errors

`VersionMismatch(8)` means the embedded migration bytes differ from the checksum
recorded when migration 8 was applied. A CRLF checkout of that migration can cause
this even when the SQL text is unchanged. Migration files are pinned to LF; rebuild
with `cargo build --locked`, then retry `make dev`. No database reset or checksum
rewrite is needed for the original LF migration history.

If the error persists, stop and identify the exact migration source originally
used for that database. Preserve the database and its migration metadata; do not
delete migration rows or bypass checksum validation. Development agents must not
inspect a personal database to investigate this.

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

## GitHub checks and dev images

[The workflow](.github/workflows/dev.yml) runs `make check` for pull requests
targeting `dev` or `main` and pushes to either branch, using the pinned
Rust/Node/pnpm versions and Chromium with its system dependencies. Failed browser journeys retain synthetic
traces/screenshots as GitHub artifacts for seven days. Other branches and release
tags do not publish images. Browser failures remain blocking on both branches.

`main` runs the same full checks without publishing images or deploying. To block
untested changes before they reach `main`, configure a GitHub branch protection
rule requiring pull requests and the **Full checks** status, and disallow bypass
if the restriction should also apply to administrators. A workflow by itself
reports failures after a direct push; it does not prevent that push. These
repository settings are separate from this workflow and have not been configured
by this change. See [GitHub's protected branch documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).

After successful checks on a `dev` push, native GitHub-hosted x86 and ARM runners
build `linux/amd64` and `linux/arm64` images. Each image must start against a fresh
disposable database, serve the API and static UI, create a synthetic user, and
restart successfully. The smoke test uses no network and cleans up its container
and anonymous data volume. Both images must pass before the combined tags publish:

- `ghcr.io/<owner>/<repository>:dev` follows the latest successful run.
- `ghcr.io/<owner>/<repository>:sha-<full-commit-sha>` identifies the checked commit.

Architecture-specific `sha-<full-commit-sha>-amd64`/`-arm64` tags are staging images;
one can exist even if the other architecture fails. Use the combined image for
deployment. The publish job summary includes its registry digest. Pin that digest
for a reproducible deployment; tags can move when a workflow is rerun. No `latest`
or release tag is published yet. Docker picks the matching architecture when
pulling the combined image; the Pi needs a 64-bit OS for `linux/arm64`.

The workflow publishes to GitHub Container Registry using its `GITHUB_TOKEN`,
with package-write permission restricted to image jobs. Repository/organization
policy must allow those permissions and the referenced actions. Existing packages
must grant this repository Actions access. Package visibility is managed in
GitHub; private images require separate read credentials on the deployment host.
No Pi credentials or deployment network access belong in CI.

Deployment remains an explicit owner action in the separate private runtime.
Its Compose configuration should select `image: ghcr.io/<owner>/<repository>@sha256:<digest>`
instead of building from source, retain its persistent `/data` volume, and set its
actual HTTPS origin/secure-cookie settings as described below. CI neither updates
the private instance nor reads its data. Use Glances and normal manual workflows
for initial performance feedback; no load-test infrastructure is required.

Workflow syntax and native/container test results must be distinguished from an
actual GitHub run: runner availability, registry permissions, image pushes, and
multi-platform assembly are verified only when the workflow runs there. Build
durations appear in Actions; there is no measured build-time or Pi runtime budget
yet. The first deployment is still pending export/reimport and private-runtime
setup.

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

### Monthly budgets

Open **Budgets** for the **Saved plan vs actual** overview: planned, actual
expenses and remaining per category. Choose **Edit monthly plan** to set targets,
reserve savings and name expected income.
Match recorded income to expectations to reduce the pending amount without
counting it twice. A blank target is Not set; zero is an explicit target. Copy a
previous plan to prepare another month; copying clears receipt links/completion.
Save applies the draft. Figures use the saved plan and the visible Recorded through
date, and describe monthly spending room rather than a bank balance. See
[monthly budgets](docs/specs/planning.md). Existing data is preserved by migration.
