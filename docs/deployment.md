# Deployment

This guide owns container builds, CI image delivery and private runtime operations.
Read [privacy and isolation](privacy.md) before using real data. Application and
portability behavior is defined in the [feature specs](specs/README.md).

## Local container build

Requirements: Docker Engine with the Compose and Buildx plugins. Check:

```sh
docker version
docker compose version
docker buildx version
```

Install the engine and its plugins using the
[official distribution instructions](https://docs.docker.com/engine/install/).
Keep the engine and plugins on the same package source.

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

The supplied [compose.yaml](../compose.yaml) binds the app to loopback. SQLite lives
in the `coinz-data` named volume mounted at `/data`, independently of the container.
`docker compose down` preserves it. Adding `--volumes` removes its data.
Native development uses a separate database; see [development](development.md).

To update after source changes, run `docker compose up -d --build`. Migrations run
on startup. Before v1.0, schema compatibility across development changes is not
guaranteed; take a backup before updating data you want to keep.

## GitHub checks and dev images

[The workflow](../.github/workflows/dev.yml) runs `make check` for pull requests
targeting `dev` or `main` and pushes to either branch, using the pinned
Rust/Node/pnpm versions and Chromium with its system dependencies. Failed browser
journeys retain synthetic traces/screenshots as GitHub artifacts for seven days. Other branches and release
tags do not publish images. Browser failures remain blocking on both branches.

`main` runs the same full checks without publishing images or deploying. To block
untested changes before they reach `main`, configure a GitHub branch protection
rule requiring pull requests and the **Full checks** status, and disallow bypass
if the restriction should also apply to administrators. A workflow by itself
reports failures after a direct push; it does not prevent that push. These
repository settings are separate from the workflow; inspect them in GitHub to
confirm the active policy. See [GitHub's protected branch documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).

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
actual origin/secure-cookie settings as described under [private runtime](#private-runtime).
CI neither updates the private instance nor reads its data. Use Glances and normal manual workflows
for initial performance feedback; no load-test infrastructure is required.

The workflow definition describes intended checks and publishing. Verify that the
run for your selected commit actually completed and published its combined image.
Build durations appear in Actions; no Pi runtime performance measurements have
been collected yet.

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

## Private runtime

Keep the same image and a persistent SQLite directory on the Pi or homelab host's
local disk.
Place an HTTPS reverse proxy in front of the app, set `COINZ_ORIGIN` to its actual
browser origin, and set `COINZ_COOKIE_SECURE=true`. Adjust Compose networking so
that proxy can reach the container; the supplied port mapping is loopback-only.
The origin and cookie settings must match how you access the app.

The homelab repo can own deployment overrides and image selection, while this
repo owns application code and its Dockerfile. A sibling checkout or a separately
built image is sufficient initially; a Git submodule is not required.

## First Pi trial

The application has passed the owner-run local checks; deployment on the Pi and
its runtime performance remain unverified. CSV/settings portability and the clean
schema baseline are implemented. The remaining deployment slice is to prepare
the owner-controlled runtime configuration and execute this trial:

1. Confirm the Pi uses a 64-bit OS suitable for the published `linux/arm64` image.
2. Select the successful `dev` publish run and pin the combined image digest.
   Keep the private Compose configuration outside the agent-accessible checkout;
   use `image:` instead of the development configuration's `build:`.
3. Configure persistent `/data` storage and the intended access route, exact
   `COINZ_ORIGIN`, and cookie security. See [private runtime](#private-runtime).
4. Create a user, then test login, a synthetic entry/import, reports, settings/CSV
   downloads and a container restart. Confirm that the records survive restart.
5. Watch Glances during normal manual workflows. Record the image digest, Pi/OS,
   storage and observed responsiveness; heavy load tests are deferred.
6. After the synthetic smoke check, the owner can begin the private real-data
   trial and collect feedback. The development agent continues using synthetic
   fixtures only.

There is no automated deployment to the Pi, production image publishing from
`main`, or measured build/runtime performance target yet.

## Updates and recovery

Choose and deploy a new pinned image explicitly. Before v1.0, schema compatibility
across development changes is not guaranteed. Startup never resets data silently;
the earlier pre-baseline history requires an explicit reset. For a disposable
native development database, use the [development reset procedure](development.md#development-schema-baseline-and-reset).

For a portable copy, download settings JSON and the entries CSV from Settings.
Restore settings into an empty account before importing entries. These files omit
credentials and original import history; the [portability contract](specs/data-portability.md)
owns the limits and restore semantics. Full database backup tooling remains deferred.

For a straightforward backup, stop all coinz processes that use the database and
copy the complete data directory/volume, including any SQLite WAL sidecars. Keep
the backup private. Restore into an empty data volume with ownership writable by
UID/GID `10001:10001`, then start the matching app version. Test restore before
relying on homelab data. Do not copy only a running SQLite main file.

