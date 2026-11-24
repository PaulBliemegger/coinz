# coinz

A personal finance notebook: clear spending insights, warm colors, and occasional
dry humor. Rust/Axum serves a static SvelteKit interface and stores data in SQLite.
One deployed container serves both the API and frontend.

coinz helps you understand recorded spending and make monthly plans from manual
entries or CSV imports. It is a self-hosted app for personal use, with local
accounts and no external bank connection.

The repository contains:

- `backend/`: Rust API, financial rules, SQLite schema and integration tests.
- `frontend/`: Svelte/TypeScript interface, unit tests and browser journeys.
- `docs/`: product scope, feature contracts and development/deployment guides.
- `examples/`: synthetic import files and a walkthrough.
- `scripts/`: development launcher and CI container smoke test.

## What works

- Local users, cookie sessions and isolated private data.
- Manual income/expenses, optional sources, inline editing and filtered entries.
- Categories, coloured labels and local description rules.
- CSV import profiles, guided mapping, duplicate review and protected batch removal.
- Monthly/yearly reporting, category spending and Insights charts.
- Monthly budgets, savings reservations and expected-income receipt matching.
- Per-user entries CSV and versioned settings JSON export/restoration.
- Light/dark appearance and responsive desktop/phone workflows.

Reports describe recorded entries, not live bank balances. EUR is the only
currency. Bank sync, transfer/refund semantics, recurring plans and file-managed
YAML profiles remain deferred. See [product scope](docs/product.md) and the
[feature specifications](docs/specs/README.md) for boundaries and behavior.

## Docker setup

Requirements: Docker Engine with the Compose and Buildx plugins. From the
repository root:

```sh
docker compose build
docker compose run --rm coinz create-user wabo
docker compose up -d
```

Choose your own username and password when prompted, then open
[http://localhost:3000](http://localhost:3000) using that exact hostname. The
supplied Compose configuration builds from source and binds to loopback.
SQLite data persists in the `coinz-data` named volume mounted at `/data`.

Stop the app with `docker compose down`; the volume is retained. Update a local
source build with `docker compose up -d --build`.

For published images, Pi configuration, HTTPS and updates/recovery, see
[deployment](docs/deployment.md). Read [privacy](docs/privacy.md) before using real
data; the development agent must stay separate from the private runtime.

## Local development

Requirements: Bash, Make, C build tools for SQLite, rustup, Node **24.22.3** and
pnpm **11.25.0**. Rust is pinned by [rust-toolchain.toml](rust-toolchain.toml).

```sh
npm install --global pnpm@11.25.0
make setup
cargo run --locked -- create-user wabo
make dev
```

Choose your own username and password when prompted; there are no default
credentials. Open [http://localhost:5173](http://localhost:5173) using that exact
hostname. Native development stores its database under `data/` by default.

Try **Import CSV** with the checked-in [synthetic demo](examples/imports/README.md).
The walkthrough includes profile mappings, rules, expected totals and an overlap
file. Use synthetic data in the development checkout.

For prerequisites, browser debugging and disposable database resets, see
[development](docs/development.md). For container builds and published images,
see [deployment](docs/deployment.md).

## Checks

```sh
make quickcheck      # Formatting and frontend lint
make browser-install # One-time Chromium download
make check           # Lint, tests, type checks, production builds and browser journeys
```

The owner has reported passing local checks after the clean schema reset.
Detailed evidence and environment limits live in
[verification status](docs/development.md#verification-status). Pi deployment and
performance are still unverified; the [first Pi trial](docs/deployment.md#first-pi-trial)
is the next deployment step.

## Keeping your data

**Settings → Your data** offers entry CSV and settings JSON downloads. To restore
into a fresh installation, create your account, restore settings into the empty
account, then import the entries CSV. These are portable copies; credentials and
original import history are excluded. See [data portability](docs/specs/data-portability.md).

Before entering real finances, read [privacy and development isolation](docs/privacy.md).
Keep the private runtime, financial data, logs and authenticated browser separate
from the development agent. Repository rules and ignore files do not enforce
that isolation on their own.

## Documentation

Each document owns one kind of information. Link to it rather than copying its
rules into another guide.

| Document | Purpose |
| --- | --- |
| [Development](docs/development.md) | Local setup, checks, browser debugging, schema resets and verification status |
| [Deployment](docs/deployment.md) | Container images, CI delivery, configuration and private runtime operations |
| [Product](docs/product.md) | Goals, boundaries and next steps |
| [Feature specs](docs/specs/README.md) | Observable behavior, API contracts and acceptance coverage |
| [Architecture](docs/architecture.md) | Structure and material technical decisions |
| [Design](docs/design.md) | Visual and interaction principles |
| [Privacy](docs/privacy.md) | Private runtime and development isolation |
| [Feature workflow](docs/feature-workflow.md) | Small implementation slices, reviews and temporary handovers |
| [Agent instructions](AGENTS.md) | Standing collaboration and coding rules, with scoped backend/frontend guidance |

[Makefile](Makefile) owns executable development/check targets. Detailed feature
behavior belongs in its spec; the README remains the entry point.
