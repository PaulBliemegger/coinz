<script lang="ts">
  import type { ImportRow } from '#lib/api.ts';
  import { money } from '#lib/money.ts';
  let {
    row,
    selected,
    choice,
    busy,
    onselect,
    onchoice,
  }: {
    row: ImportRow;
    selected: boolean;
    choice: string;
    busy: boolean;
    onselect: (checked: boolean) => void;
    onchoice: (choice: string) => void;
  } = $props();
</script>

<article
  class="review-row"
  class:attention={row.status === 'invalid' ||
    row.status === 'conflict' ||
    (row.status === 'review' && !choice)}
>
  <div class="row-main">
    <input
      type="checkbox"
      aria-label={`Include ${row.description || 'invalid row'}`}
      checked={selected}
      disabled={busy || !(row.status === 'new' || (row.status === 'review' && choice === 'keep'))}
      onchange={(event) => onselect(event.currentTarget.checked)}
    />
    <div>
      <h4>{row.description || 'Missing description'}</h4>
      <p class="muted">{row.date}</p>
      {#if row.status === 'known'}<span class="badge">Already imported</span>
      {:else if row.status === 'invalid'}<span class="badge warning">Couldn’t read this entry</span>
      {:else if row.status === 'conflict'}<span class="badge warning">Conflicting bank ID</span>
      {:else if row.status === 'review'}<span class="badge" class:warning={!choice}>
          {choice.startsWith('match:')
            ? 'Matched to existing entry'
            : choice === 'keep'
              ? 'Separate purchase'
              : 'Possible duplicate'}
        </span>{/if}
    </div>
    <span class="amount">{row.amount_cents ? money(row.amount_cents, true) : 'Not validated'}</span>
  </div>
  {#if row.error}<p class="row-error">{row.error} Left out for now.</p>{/if}
  {#if row.status === 'conflict'}<p class="row-error">
      This bank ID has different or repeated data. Correct the CSV to include this row.
    </p>{/if}
  {#if row.status === 'review'}
    <label class="decision">
      Duplicate decision for {row.description}<select
        value={choice}
        disabled={busy}
        onchange={(event) => onchoice(event.currentTarget.value)}
      >
        <option value="">Leave for later</option><option value="keep">
          Keep as a separate purchase
        </option>
        {#each row.candidates as candidate (candidate.id)}<option value={`match:${candidate.id}`}>
            Match: {candidate.date} · {candidate.description} · {money(candidate.amount_cents)}
          </option>{/each}
      </select>
    </label>
  {/if}
  <details>
    <summary>Import details</summary>
    <p class="muted">
      CSV record {row.record}{row.external_id ? ` · Bank ID ${row.external_id}` : ' · No bank ID'}
    </p>
  </details>
</article>

<style>
  .review-row {
    padding: 1rem 0;
    border-bottom: 1px solid var(--line);
    overflow-wrap: anywhere;
  }
  .attention {
    background: var(--review-fill);
    padding: 1rem;
    border-radius: 0.7rem;
    margin-bottom: 0.75rem;
    border: 0;
  }
  .row-main {
    display: grid;
    grid-template-columns: 20px minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: start;
  }
  input[type='checkbox'] {
    width: 19px;
    min-height: 19px;
    margin-top: 0.4rem;
    accent-color: var(--accent);
  }
  h4 {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
  }
  .amount {
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
    max-width: 100%;
  }
  .badge {
    display: inline-block;
    background: var(--status-fill);
    color: var(--status-ink);
    font-size: 0.875rem;
    padding: 0.15rem 0.5rem;
    border-radius: 1rem;
    margin-top: 0.35rem;
  }
  .warning {
    background: var(--review-fill);
    color: var(--review-ink);
  }
  .decision,
  .row-error,
  details {
    margin-top: 0.75rem;
  }
  details {
    font-size: 0.875rem;
    color: var(--muted);
  }
  summary {
    min-height: 36px;
  }
  .row-error {
    font-size: 0.875rem;
    color: var(--review-ink);
  }
  @media (max-width: 420px) {
    .amount {
      grid-column: 2;
    }
  }
</style>
