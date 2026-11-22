# Feature chats and handovers

Use one chat per coherent feature, with one active implementation slice at a time.
A new chat does not require a new branch or worktree for this solo workflow.
Use worktrees only when deliberately running independent work in parallel.

## Start

Give the chat a concrete user outcome and point it at the relevant feature spec.
Ask only a few questions where answers change scope. For visual changes, agree a
small synthetic design before implementation. Record approved behavior and
acceptance examples in `docs/specs/<feature>.md`, marked Proposed until shipped.

Keep reusable approved designs under `docs/designs/<feature>/` when needed, using
relative links from the spec. Include desktop/phone and light/dark references
when they materially affect the design. Never use screenshots of real finances.
Do not rely on another chat's memory or a machine-specific preview path.

## Implement and finish

The implementation chat reads AGENTS.md plus the relevant specs, implements a
small working slice, adds meaningful regressions, and updates affected docs.
Run `make quickcheck` and `make check`; distinguish environment failures from
failed assertions. Inspect synthetic desktop/phone screenshots for UI changes.

Finish with a deliberate code review of the resulting diff: ownership/CSRF,
financial/date rules, data preservation, races/stale UI, accessibility, tests and
documentation. Fix findings and rerun affected checks. A self-review is useful but
is not an independent review; a fresh review chat is optional for risky changes.
No commits, branches, pushes, or merges without authorization.

## Handover only at a useful boundary

Prefer the same chat through design → implementation → verification → review.
Start another when moving to a new feature, needing a fresh review, or handing
off unfinished work with a clear next action. Avoid a chat per tiny step.

The feature spec remains the durable product record. Put unfinished operational
context in one temporary `docs/handoffs/<feature>.md`, linking to the spec instead
of copying it. Include:

- Goal, current state, remaining acceptance cases, and explicit non-goals.
- Relevant source/spec/design paths and material decisions still unresolved.
- Worktree/branch/base commit if applicable; whether work is committed.
- Exact checks run, results, blocked checks, and synthetic screenshot locations.
- Known review findings and the first next action.

Delete the temporary handover when resolved. Do not keep a second roadmap or
chronological diary. Completed features need current specs and evidence, not a
permanent handover document. Store generated Playwright output in ignored
`frontend/test-results/`; copy only intentionally maintained synthetic references
into `docs/designs/`.

A starting prompt:

> Implement the agreed slice in docs/specs/FEATURE.md. Read AGENTS.md and the
> linked design. Keep the scope small; clarify only material product decisions.
> Use synthetic data. Update the authoritative docs, run required checks, inspect
> the UI, review the diff, fix findings, and report any remaining limitations.
> Do not make Git changes unless explicitly authorized.

## Skills: add one when repetition earns it

Keep standing conventions in AGENTS.md and behavior in specs. A future small
`coinz-feature` skill could package the start/verify/review/handover routine above,
linking to this document rather than duplicating it. A separate `coinz-review`
skill is worthwhile only if a distinct review checklist repeatedly proves useful.
Do not add orchestration scripts, plugins, or mandatory multi-agent stages now.
Repo-local reusable skills live under `.agents/skills/<name>/SKILL.md`; see
[official customization guidance](https://learn.chatgpt.com/docs/customization/overview)
and [building skills](https://learn.chatgpt.com/docs/build-skills).
