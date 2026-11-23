# Design philosophy

Status: established direction. [Feature specs](specs/README.md) own detailed
interaction rules and implementation status; the
[README](../README.md#verification-status) records current verification limits.

## Warm, quiet, useful

- Lead with the person's task and financial meaning. Transaction description,
  date, and amount come before import record numbers, bank IDs, and filenames.
- Use cream paper in light mode and warm brown surfaces in dark mode, with
  restrained terracotta actions. System appearance is the default. Keep entry
  rows compact while retaining comfortable spacing between sections.
  Serif headings give character; readable sans-serif text and aligned tabular
  amounts support scanning. Reuse existing tokens and controls.
- Color helps identify states, accompanied by clear words. Use subdued green for
  ready/matched, amber for review, and red for errors/destructive actions. Ordinary
  expenses need no red warning treatment. Never imply spending is moral failure.
- Reveal detail when needed: provenance details, raw import identifiers, and
  technical record references belong in expanded details or actionable errors.
  Keep essential amounts, scope, errors, and duplicate state visible.
- Show contextual controls when they are useful. One primary action per step;
  bulk controls appear after selection. Avoid permanent dashboard toolbars.
- Humor is occasional and low stakes. Keep confirmations, errors, duplicate
  decisions, and financial assumptions literal. No streak pressure or rewards
  tied to income/spending amounts.
- Support keyboard and phone use with labeled controls, visible focus, adequate
  contrast, touch targets, and layouts that reflow instead of shrinking text.

## Import and entry interactions

Keep imported transactions readable as entries, rather than exposing the CSV's
row structure as the primary interface. Put review decisions before ready items,
and keep the final action within reach. The
[CSV review spec](specs/csv-import.md#review-interface) owns grouping, selection,
and duplicate decisions; [import history](specs/import-history.md) owns removal
previews and edit protection.

Entries should read like a notebook at rest. Direct field activation avoids an
extra edit mode, and whole-row feedback makes selection clear. Motion should
support that feedback without moving the text or controls. See
[ledger interactions](specs/manual-ledger.md#direct-editing-selection-and-provenance)
for selection, animation, editing, and deletion rules.

Home answers the quick overview question; Entries supports exploring the records.
Keep the displayed period prominent and the scope of actions explicit. The
[reporting spec](specs/reporting.md) owns dashboard layout, the centered month
picker, filters, and pagination.

The approved [Guided selection design](specs/import-profiles.md#approved-interaction-design)
extends import setup with table-header mapping and description editing in review.
It is implemented; browser/visual verification remains blocked. The spec records the interaction so future work
does not depend on a chat-only prototype.

## Small view preferences

A considered default is part of the product. Keep a compact preference entry
point; add an option only after a recurring need appears. Appearance and selection
visibility are small personal preferences, not a layout or theme builder.
Essential touch and keyboard controls must remain available regardless of visual
hiding. The [ledger spec](specs/manual-ledger.md#appearance-and-user-menu) owns the
current user menu and appearance behavior; selection visibility is defined in its
[interaction section](specs/manual-ledger.md#direct-editing-selection-and-provenance).
Do not add nonfunctional controls for future settings or export.

## Evolution

Compare concrete designs with synthetic data, pick the interaction that feels
clearest, and implement one slice at a time. Update the affected feature spec
when a proposal becomes behavior; remove superseded proposals here instead of
collecting a second roadmap. Favor a quiet ledger over configurable layouts or
an elaborate design system.
