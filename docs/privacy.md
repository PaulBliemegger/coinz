# Private finance data and development isolation

Status: required deployment boundary; not enforced by the repository alone.
The current development session has broad filesystem read access. Its code
changes and instructions do not establish an absolute data-access guarantee.

## Requirements

Real transactions, balances, bank exports, credentials, logs, screenshots, browser
sessions, and backups must stay on the owner's laptop and out of agent context.
Code development uses synthetic fixtures only. Never treat replacing names in a
real export as proof that its financial content has been anonymized.

## Recommended separation

Use a dedicated development VM containing only the source checkout and synthetic
fixtures. Give the development agent no shared private directories, host Docker
socket, production browser profile, production credentials, or network route to
the real application. Docker used inside that VM must be a separate daemon.
A container connected to the host Docker socket does not provide this separation.

Run the private application under a separate host user that the development
identity cannot impersonate. That user owns its data directory, imports, backups,
configuration, and private browser profile. Keep the source checkout shared with
the agent free of real data; private deployment configuration belongs outside it.

The owner reviews the built source/image and explicitly deploys it to the private
instance. Do not let an agent-controlled development watcher or rebuild command
update the application while it can process real data. Source isolation alone
cannot prevent newly deployed code from reading the database it needs to serve.

For no financial-data egress, enforce outbound restrictions independently of
application code: isolate the runtime's network through host/container firewall
policy and control the browser's outbound destinations too. A frontend can cause
browser traffic even when the backend has no internet access. Keep real data out
of cloud backups, crash reporters, browser sync/extensions, and support uploads.
Allow dependency downloads/builds in the synthetic development environment.

These measures address the development agent and accidental network egress. They
are not an absolute guarantee against host administrators, a compromised OS,
malicious dependencies, or owner-approved changes. If absolutely no information
may leave the laptop, including code/prompts/tool results, use offline local
inference and local-only tools rather than this cloud-backed agent session.

## Additional sandbox controls

As defense in depth, deny reads/writes to exact private directories in the agent's
actual permission policy; workspace-write alone may still allow broad reads.
Do not grant outside-sandbox escalation, host Docker access, browser/Computer Use
access to the private app, or a shared authenticated profile. Filesystem policy
alone does not control independent browser/connector surfaces. Configure those
separately and test access using an artificial canary, never a real finance file.

A broad localhost network grant can also expose the real app on a different
port; local development networking is not separation from a same-host private app.

A separate OS user can help, but it is not a boundary if the agent can sudo,
impersonate that user, read backups, or control the host daemon. For the stronger
separation above, the development VM must not have those capabilities.

See [Codex permissions](https://learn.chatgpt.com/docs/permissions) for exact deny
paths, enforcement scope, and independent tool surfaces. Repo `AGENTS.md`,
`.gitignore`, and `.dockerignore` are workflow/build controls, not OS access rules.
Disabling model training does not prevent content being transmitted for inference.

## Current source review

The reviewed app-owned frontend requests its same-origin `/api`; no explicit
analytics, third-party fonts, or remote runtime integrations were found in the
reviewed frontend/backend source. Dependency installation, Docker pulls, and
build-time package checks use the internet. This was not a dependency/OS audit
and must not be interpreted as proof of zero egress.

`compose.yaml` publishes only to loopback, but its default Docker network does
not enforce outbound isolation. Its named data volume is outside the source tree,
but an agent with host Docker daemon access can potentially access it. Native
`data/` is inside the checkout and therefore unsuitable for real data under the
current development permissions. Runtime isolation must be configured separately.

## Repository rules

- Never open or query personal databases, exports, private logs, or backups.
- Never attach to the real app/container or authenticated browser to debug it.
- Never collect screenshots or traces of real finance data.
- Test with fresh temporary databases and wholly synthetic rows.
- Keep private paths and volumes out of builds/version control; avoid broad scans
  that could cross into private data. Scope searches to source and docs.
- Report verification limits honestly. Privacy boundaries must not be weakened
  simply to make browser tests easier to run.
