# Agent test environments

[Development](development.md#tests-and-checks) owns test commands, fixture isolation
and the current [verification status](development.md#verification-status). This
page describes how to distinguish environment limitations from application failures.

Playwright starts a local Rust server on `127.0.0.1:4100` and launches Chromium.
Both local listening and browser processes must be permitted by the execution
environment. A server startup `PermissionDenied` or `Operation not permitted`
occurs before browser assertions; report it as an unavailable check, not as a
passing suite or evidence that an application assertion failed.

When diagnosing an unavailable run:

1. Identify whether the failure is server startup, browser launch or an assertion.
2. Check the active environment's permissions and required browser installation.
3. If the environment cannot run the suite, use an owner-run local check or CI
   and distinguish reported results from checks the agent actually executed.
4. After an authorized environment change, rerun the focused journey and inspect
   synthetic screenshots before claiming UI verification.

Machine-specific policy settings and temporary approval decisions do not belong
in project-wide prerequisites. Configure permissions through the owning
client/environment; a repository edit does not grant them.

Keep the [privacy boundary](privacy.md) intact. Tests use temporary synthetic data;
host Docker access and access to a private runtime are not test prerequisites.
An explicitly authorized synthetic browser session can support UI review, but it
does not replace executing the complete Playwright journeys.
