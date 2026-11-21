# Specifications

These documents define observable behavior and examples, not every function or
screen. Read [product scope](../product.md) for goals and sequencing, and
[architecture](../architecture.md) for implementation boundaries.

| Spec | State | Scope |
| --- | --- | --- |
| [Manual ledger](manual-ledger.md) | Implemented; full verification pending | Identity, optional sources, entries, period totals, preserved legacy data |
| [Planning](planning.md) | Proposed; not implemented | Monthly assumptions, commitments, one-off scenarios, optional later calculator |

Imports, labels, and richer insights remain product direction. Write their detailed
specs just before implementation using useful questions and wholly synthetic source
samples. Account balances, reconciliation, and historical balance reconstruction
are outside the core. Snapshot APIs/UI have been retired; legacy data remains stored.

## Keeping specs useful

- One authoritative home per rule. Link instead of copying it across documents.
- Each feature spec states its status, observable behavior, important boundaries,
  acceptance examples, test locations, and unresolved questions.
- Update the spec and relevant tests in the same change as behavior. Review old
  assertions and remove superseded decisions rather than appending corrections.
- UI layout and copy can evolve without rewriting financial behavior. Record a
  screen decision only when it changes a meaningful user interaction or promise.
- Split a document when independently changing features make it hard to navigate.
  Move content to named feature specs, preserving links; do not copy it.
- Add an architecture decision record only for a consequential tradeoff whose
  rationale would otherwise be lost. Do not create one for every library or edit.
- Keep source/test details in code and setup commands in the README/Makefile.
  Specs describe what should happen and why it matters.
