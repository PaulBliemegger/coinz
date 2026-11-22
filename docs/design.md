# Design philosophy

Status: established direction. The agreed import review, direct editing, selection,
and view preference are implemented; browser/visual verification remains pending.
[Feature specs](specs/README.md) own detailed interaction and financial rules.

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

The approved Review tray groups attention-needed imports before ready entries,
with a persistent bottom import action. Selection means a deliberate subset, and
leaving a row out must not silently ignore it forever. See the authoritative
[CSV behavior](specs/csv-import.md#review-interface).

Entries look like read-mode content at rest, with direct field activation and
contextual selection controls. Bulk deletion confirms beside the stationary bin;
users do not have to leave the toolbar. Source and entry method remain distinct, on a compact secondary line. Import
detail foldouts are removed until a clearer need emerges. A prominent period
picker and period-specific selection labels make the scope visible.
See [ledger interactions](specs/manual-ledger.md#direct-editing-selection-and-provenance).

## Small view preferences

The username dropdown offers Appearance (System / Light / Dark), Settings, and
Log out. The separate Settings sidebar holds **Always show selection controls**, off by default
on devices with hover. It changes visibility only; direct editing stays available
in both views. Touch and keyboard accessibility override visual hiding as above.
Keep one compact Settings entry point for present and future preferences. Export
is future work; do not add a nonfunctional export control. Avoid a layout builder
or a preference for every spacing/color decision. A considered default
is part of the product. Appearance defaults to System; explicit overrides are
small local preferences, not a configurable theme builder. Add another option only after a recurring need appears.

## Evolution

Compare concrete designs with synthetic data, pick the interaction that feels
clearest, and implement one slice at a time. Update the affected feature spec
when a proposal becomes behavior; remove superseded proposals here instead of
collecting a second roadmap. Favor a quiet ledger over configurable layouts or
an elaborate design system.
