# Import profiles implementation handoff

Status: ready for a future implementation chat. Design/specification work only;
the feature is not implemented. Remove this temporary handoff after the slice is
implemented and verified. The [feature spec](../specs/import-profiles.md) is the
authoritative product/design definition; do not maintain a second copy here.

## Current state and first action

The user approved **Guided selection**, with one Description field and editable
descriptions during review. Start by reading the required docs and inspecting
the current source and working-tree diff, then implement the agreed flow. This
checkout contains other uncommitted work; preserve it. This chat created no Git
commits, branches, remotes, or worktrees and made no production source changes.

Relevant implementation boundaries are `backend/src/imports/`, the ledger's
description validation, `frontend/src/lib/features/imports/`, the API client, and
the existing import history/provenance tests. Read the scoped AGENTS.md files
before editing backend/frontend code. The spec owns acceptance examples and
deferred work, including transfer/refund semantics; do not reopen settled design
choices or expand this slice into unrelated finance features.

## Design reference and verification

The approved prototype remains outside the repository at:

```text
/home/wabo/.codex/visualizations/2026/11/22/01a204de-a0cf-7c03-b3b6-85b7a174a79a/coinz-csv-table-mapping.html
```

Use only its **Guided selection** variant. The alternative Header menus and older
two-text-field prototypes are not implementation targets. The prototype is a
synthetic design reference, not production code. It reads no real CSVs and saves
no actual profiles or entries. Do not put the HTML back into `docs/` or copy the
prototype wholesale into the frontend. If this optional local reference is
unavailable, the spec's approved-design section fully describes the interaction.

Checks performed: HTML fragment/size/whitespace validation, `node --check` on the
extracted script, and a Node VM interaction check using synthetic event/DOM stubs
for sequential mapping, both mapping variants, description editing, exact totals,
duplicate choices, and draft-discard confirmation. These are prototype logic
checks, not real browser or production acceptance evidence.

Browser rendering remains unverified: Chromium failed to launch in the sandbox
with `Operation not permitted`; the user declined an outside-sandbox check. Do not
retry that escalation automatically or access a private browser/app as a fallback.
No `make quickcheck`/`make check` run was needed for this documentation/design-only
step. Run required checks for the later implementation and distinguish assertions
from environmental blockers. No independent implementation review has occurred.

Remaining engineering choices include bounded parsing presets, the profile update
and import-interpretation storage mechanism, and exact API shape. These are ordinary
implementation decisions within the spec. First inspect the existing parser,
preview/commit logic, import history, and ownership tests; use only synthetic data.

## Prompt for the implementation chat

Implement the agreed CSV import-profile slice in this coinz checkout. Read
AGENTS.md, docs/product.md, docs/architecture.md, docs/design.md,
docs/specs/README.md, docs/specs/import-profiles.md, the linked current CSV/ledger
and import-history specs, docs/privacy.md, and the relevant scoped AGENTS.md files.
Use this handoff for operational context; the feature spec owns behavior.

Implement the approved Guided selection flow: choose/create a reusable owned
profile, map columns by clicking headers in a real CSV sample table, then continue
to the existing selected-row import review. Use one Description field; a CSV name
or counterparty column can supply it. Support lightweight description editing in
review and require nonempty descriptions on saved entries. Keep manual entry
simple. Preserve original import evidence and existing duplicate, user-edit,
selection, ownership, atomicity, CSRF, and import-history guarantees. Retain exact
integer cents and explicit date/number interpretation.

Keep this one coherent slice. File-managed YAML/GitOps profiles, entry export and
restore, separate counterparties, profile management/deletion screens, extra
amount layouts, and transfer/refund classification remain deferred. Do not claim
arbitrary bank-format support from column mapping alone. Make routine engineering
decisions without reopening approved UX; ask only if a new choice changes scope.

Use source/docs and wholly synthetic fixtures with temporary databases only.
Preserve unrelated working-tree changes. Add meaningful parser, persistence,
ownership, preview-edit/reimport, and browser regressions. Run make quickcheck and
make check, visually inspect synthetic desktop/phone and light/dark views when
available, review the resulting diff, fix findings, and update authoritative docs.
Report any blocked checks and remaining limitations honestly. Do not repeat the
declined browser escalation automatically. No commits, branches, remotes, pushes,
or deployment unless separately requested. Delete this temporary handoff when the
implementation is complete.
