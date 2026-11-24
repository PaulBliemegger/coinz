# Feature work and handovers

Keep one active implementation slice with a concrete user outcome. Continue in
the same chat while its context remains useful; a fresh chat can help with a new
feature, an independent review or an unfinished handover. A new chat does not
require a branch or worktree. Follow [AGENTS.md](../AGENTS.md) for collaboration,
Git authorization, privacy and verification rules. Do not create commits, branches,
pushes or merges without user authorization. Use worktrees only for deliberately
independent parallel work.

## Define and implement

Read the relevant feature spec and clarify only choices that change scope.
Record approved behavior and concrete acceptance examples in that spec; mark
proposals and deferred work explicitly. For UI changes, use the [design principles](design.md)
and concrete synthetic examples to resolve material interaction choices before
implementation. Include desktop/phone and light/dark references when they affect
the design. Keep intentionally maintained design
references under `docs/designs/<feature>/` when needed, linked from the spec.
Never rely on screenshots of real finances or a machine-specific preview path.

Implement a small working slice, add meaningful regression coverage and update
affected documentation. Use the [development guide](development.md#tests-and-checks)
for commands and report executed, unavailable and owner-reported checks separately.
Documentation-only edits need link/anchor and source-consistency checks, as defined
in the root agent instructions.

Review the resulting diff for ownership/CSRF, exact money/date handling, data
preservation, stale state, accessibility and test coverage as applicable. Fix
findings and rerun affected checks. A self-review does not constitute an
independent review; a fresh review is optional for risky changes.

## Hand over unfinished work

The feature spec is the durable product record. Only when work is unfinished,
put operational context in one temporary `docs/handoffs/<feature>.md`:

- Goal, current state, next action, explicit non-goals and remaining acceptance cases.
- Relevant source/spec/design paths and unresolved decisions.
- Branch/worktree, base commit and current commit state, when applicable.
- Exact checks/results, known findings and synthetic screenshot locations.

Link to existing rules instead of copying them. Delete the handover when resolved.
Generated Playwright output belongs in ignored `frontend/test-results/`; copy only
intentionally maintained synthetic references into the design documentation.
Keep standing conventions in agent instructions and behavior in specs. Add extra
workflow tooling only for a demonstrated recurring need.
