# Agent browser-test permissions

## Observed limitation

This chat's active managed sandbox has restricted networking. A Python probe failed
at `socket.socket()` with `PermissionError: Operation not permitted`, before it
could bind localhost. Playwright needs a local Rust server on `127.0.0.1:4100` and
browser processes, so the existing sandbox cannot complete the browser journeys.
The earlier request for an outside-sandbox test run was declined. That permission
has not been changed or worked around. Ordinary terminal runs use their own OS
permissions, independent of the agent sandbox.

The synthetic runner bounds Tokio workers and supplies its fake password through
a temporary file descriptor, avoiding a Node piped-stdin startup stall observed in
this environment. With that fixed, the backend exits with `Operation not permitted`
at local-server startup in the sandbox. This is distinct from browser assertions.

## Sandboxed local networking

Codex permission profiles can allow networking while retaining filesystem
restrictions. With the network proxy active, allowlisting exact local literals
such as `localhost` and `127.0.0.1` allows those targets. Enabling a network profile
does not itself enable the proxy. Broader `allow_local_binding` is for cases that
also need local/private addresses resolved from allowed hostnames; it is not the
first choice for these literal-loopback tests.

Use the permission settings for the selected execution environment. Managed policy
can override user settings. Legacy `sandbox_mode` settings and newer permission
profiles are separate configurations; do not combine them. Browser process
restrictions and browser/cache filesystem access also need to permit the chosen
runner. A local network grant alone does not guarantee a browser can launch.

See the [official permissions documentation](https://learn.chatgpt.com/docs/permissions),
especially local/private networks and migration from older sandbox settings.
Do not paste a profile snippet over an existing managed configuration. The agent
cannot relax its own policy by editing a repository file.

## Example for an existing permission profile

If your client uses permission profiles, merge the following into its user
configuration, replacing `YOUR_PROFILE` with the existing selected profile name.
Retain that profile's filesystem rules. This is a configuration fragment, not a
complete profile, and cannot override a managed denial:

```toml
[features]
network_proxy = true

[permissions.YOUR_PROFILE.network]
enabled = true

[permissions.YOUR_PROFILE.network.domains]
"localhost" = "allow"
"127.0.0.1" = "allow"
```

The proxy keeps destination rules effective. The example allows local literal
targets, not package registries; install dependencies separately or add specifically
needed registry destinations. Do not add this fragment to a client still using
legacy `sandbox_mode` settings: follow its migration documentation first. Apply
changes through the owning client/environment and recheck the next session's
actual policy. These snippets are guidance, not changes the agent has applied.

## Rechecking after a policy change

1. Confirm a loopback socket can be created and bound in the new session.
2. Follow the README's browser installation/artifact build commands.
3. Run `pnpm --dir frontend test:e2e`; diagnose server startup separately from
   browser launch and test assertions.
4. Inspect desktop/phone screenshots before calling UI verification complete.

Avoid granting the test runner Docker daemon access: it uses a native Rust process
and temporary SQLite data. Registry access is needed for dependency installation;
installed browser journeys use local assets and loopback only.
