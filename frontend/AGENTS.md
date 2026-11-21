# Frontend guidelines

Read the root `AGENTS.md` first. The interface follows the product/spec documents;
these instructions cover Svelte and TypeScript implementation choices.

## Structure and components

- Routes compose screens and coordinate loading. Extract dialogs, forms, and
  independently understandable panels as their flows evolve; avoid growing one
  page into the entire application.
- Group feature-specific UI and helpers under `src/lib/features/<feature>/` when
  there is enough related code to justify a folder. Put genuinely shared visual
  primitives under `src/lib/components/` only once they are actually reused.
- Existing `src/lib/api.ts` owns the HTTP boundary and session token; `money.ts`
  owns exact display helpers. Keep shared modules small and specifically named.
  Do not create generic `utils`, global state frameworks, or a component library
  in anticipation of future needs.
- Use Svelte 5 runes and event attributes, TypeScript strict checking, and ordinary
  local state. SvelteKit 3 configuration lives in `vite.config.ts`; internal
  imports use the package's `#lib/*` mapping with explicit extensions.

## Data and interaction

- Rust owns financial calculations and authorization. Preserve cents strings at
  the API boundary and use BigInt for exact display/editing. Never convert money
  through JavaScript Number or floating-point arithmetic.
- Model loading, empty, success, and error states explicitly. Prevent duplicate
  submissions while saving and ignore superseded reads after filter changes.
- Close a form after a confirmed successful mutation; report a subsequent refresh
  failure without suggesting the mutation itself failed or should be resubmitted.
- Keep secrets/tokens out of localStorage, URLs, and logs. Use the shared API client
  for cookie credentials, CSRF headers, and error handling.
- Use native forms, controls, and dialogs with labels, keyboard support, visible
  focus, and appropriate status/error announcements. Confirm destructive actions.
- Dates represent calendar dates. Do not convert a transaction date to UTC and
  back; form defaults use the browser's local calendar date.

## Appearance

- Use the existing warm palette and CSS tokens. Keep body text at least 16px,
  normal labels at least 14px, and touch targets around 44px.
- Maintain readable contrast and layouts at narrow phone and laptop widths.
  Essential actions must work without hover. Respect reduced-motion preferences.
- Humor belongs in occasional low-stakes copy; validation and financial meaning
  remain clear. The product spec defines recorded figures and data freshness.
- Reuse established spacing, typography, forms, and button styles. Add design
  tokens when a repeated visual decision needs one; avoid per-component drift.

## Verification

From the repo root: `pnpm --dir frontend format:check`, `pnpm --dir frontend lint`,
`pnpm --dir frontend check`, `pnpm --dir frontend test`, and
`pnpm --dir frontend build`. Use `make check` for the complete suite.

- Unit-test meaningful frontend logic such as exact money formatting, not Svelte's
  own bindings. Use Playwright for complete flows against the real Rust backend.
- Exercise desktop and phone layouts and inspect screenshots for visible changes.
  Browser tests use temporary data, never a personal database.
- A sandbox/browser startup failure is an environment failure until diagnosed.
  Report it separately from application assertions and do not claim visual QA
  based only on a successful build.
