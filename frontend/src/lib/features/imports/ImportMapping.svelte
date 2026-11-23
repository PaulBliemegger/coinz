<script lang="ts">
  import { untrack } from 'svelte';
  import {
    api,
    type CsvSample,
    type ImportLayout,
    type ImportProfile,
    type ImportRow,
  } from '#lib/api.ts';
  import { money } from '#lib/money.ts';
  let {
    csv,
    sourceId,
    profile,
    onsaved,
    onback,
  }: {
    csv: string;
    sourceId: string;
    profile: ImportProfile | null;
    onsaved: (profile: ImportProfile) => Promise<void>;
    onback: () => void;
  } = $props();
  const fields = ['date', 'amount', 'description'] as const;
  type Field = (typeof fields)[number];
  const labels = { date: 'Date', amount: 'Amount', description: 'Description' };
  let layout = $state<ImportLayout>(
    untrack(() => profile)
      ? { ...untrack(() => profile)!.layout }
      : {
          date: '',
          amount: '',
          description: '',
          external_id: null,
          delimiter: ',',
          date_format: 'YYYY-MM-DD',
          decimal: '.',
          grouping: '',
        },
  );
  let name = $state(untrack(() => profile?.name ?? ''));
  let saved = $state(untrack(() => profile));
  let active = $state<Field | null>(untrack(() => (profile ? null : 'date')));
  let sample = $state<CsvSample | null>(null);
  let rows = $state<ImportRow[]>([]);
  let error = $state('');
  let reading = $state(false);
  let saving = $state(false);
  let sequence = 0;
  let hoveredColumn = $state<number | null>(null);
  let focusedColumn = $state<number | null>(null);
  function highlighted(index: number): boolean {
    const header = sample?.headers[index];
    return (
      !!active &&
      !saving &&
      !!header &&
      sample?.headers.filter((h) => h === header).length === 1 &&
      (hoveredColumn ?? focusedColumn) === index
    );
  }
  const complete = $derived(fields.every((field) => !!layout[field]));
  const count = $derived(fields.filter((field) => !!layout[field]).length);
  $effect(() => {
    const current = $state.snapshot(layout);
    const request = ++sequence;
    void read(current, request);
    return () => {
      sequence++;
    };
  });
  async function read(current: ImportLayout, request: number) {
    reading = true;
    error = '';
    rows = [];
    try {
      const result = await api<CsvSample>('/imports/sample', 'POST', {
        csv,
        delimiter: current.delimiter,
      });
      if (request !== sequence) return;
      sample = result;
      if (fields.every((f) => current[f])) {
        const preview = await api<{ rows: ImportRow[] }>('/imports/preview', 'POST', {
          csv,
          source_id: sourceId,
          layout: current,
        });
        if (request !== sequence) return;
        rows = preview.rows.slice(0, 5);
      }
    } catch (cause) {
      if (request === sequence)
        error = cause instanceof Error ? cause.message : 'Could not read this layout.';
    } finally {
      if (request === sequence) reading = false;
    }
  }
  function assign(header: string) {
    if (!active) return;
    for (const field of fields) if (layout[field] === header) layout[field] = '';
    if (layout.external_id === header) layout.external_id = null;
    layout[active] = header;
    active = fields.find((field) => !layout[field]) ?? null;
  }
  async function save(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    error = '';
    try {
      saved = await api<ImportProfile>(
        saved ? `/imports/profiles/${saved.id}` : '/imports/profiles',
        saved ? 'PUT' : 'POST',
        {
          name,
          layout,
          ...(saved ? { revision: saved.revision } : {}),
        },
      );
      await onsaved(saved);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not save profile.';
    } finally {
      saving = false;
    }
  }
</script>

<section class="mapping-workspace" aria-label="Map CSV columns" aria-busy={saving}>
  <h3>Map columns</h3>
  <p>
    Choose the columns for your entries. A name, counterparty, or reference can supply Description.
  </p>
  <div class="fields" aria-label="Transaction fields">
    {#each fields as field, index (field)}
      <button
        type="button"
        class="button field"
        data-field={field}
        class:active={active === field}
        aria-pressed={active === field}
        disabled={saving}
        onclick={() => (active = field)}
      >
        <span class="field-title">
          <span class="field-number">{index + 1}</span><strong>{labels[field]}</strong><span
            class="field-state"
          >
            {layout[field] ? 'Mapped' : 'Choose column'}
          </span>
        </span>
        <span class="field-column">{layout[field] || 'Select a CSV header below'}</span>
      </button>
    {/each}
  </div>
  <p class="mapping-prompt" data-field={active} role="status">
    {count} of 3 mapped · {active
      ? `Choose the ${labels[active]} column`
      : 'All three fields mapped. Select a field above to change it.'}
  </p>
  {#if sample}
    <p class="muted">
      Original CSV sample · {sample.sample.length} of {sample.total} rows. Empty cells say “Empty”.
    </p>
    <!-- Keyboard focus lets users scroll the sample without using a pointer. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="table-scroll"
      data-active={active}
      role="region"
      aria-label="Original CSV sample"
      tabindex="0"
    >
      <table>
        <thead>
          <tr>
            {#each sample.headers as header, i (i)}
              <th
                data-field={fields.find((f) => layout[f] === header)}
                class:column-hover={highlighted(i)}
                scope="col"
              >
                <button
                  type="button"
                  disabled={saving ||
                    !active ||
                    sample.headers.filter((h) => h === header).length !== 1}
                  onpointerenter={() => (hoveredColumn = i)}
                  onpointerleave={() => (hoveredColumn = null)}
                  onfocus={() => (focusedColumn = i)}
                  onblur={() => (focusedColumn = null)}
                  onclick={() => assign(header)}
                >
                  {header}
                  {#each fields.filter((f) => layout[f] === header) as field (field)}<span
                      class="badge"
                    >
                      {labels[field]}
                    </span>{/each}
                  {#if sample.headers.indexOf(header) !== sample.headers.lastIndexOf(header)}<span>
                      Duplicate header · unavailable
                    </span>{/if}
                  {#if active && !saving && sample.headers.filter((h) => h === header).length === 1}
                    <span class="column-action">
                      Do I describe {labels[active]}?
                    </span>
                  {/if}
                  <span class="sr-only"> column {i + 1}</span>
                </button>
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each sample.sample as row, rowIndex (rowIndex)}<tr>
              {#each row as cell, i (i)}<td
                  data-field={fields.find((f) => layout[f] === sample?.headers[i])}
                  class:column-hover={highlighted(i)}
                >
                  {cell || 'Empty'}
                </td>{/each}
            </tr>{/each}
        </tbody>
      </table>
    </div>
    <label>
      External transaction ID (optional)
      <select disabled={saving} bind:value={layout.external_id}>
        <option value={null}>No stable ID column</option>
        {#each sample.headers.filter((h) => !fields.some((f) => layout[f] === h) && sample?.headers.filter((v) => v === h).length === 1) as header (header)}<option
            value={header}
          >
            {header}
          </option>{/each}
      </select>
    </label>
  {/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  <div class="interpretation-grid">
    <details class="reading-panel">
      <summary>Reading the values</summary>
      <p class="muted">
        EUR only. Positive amounts are income; negative amounts are expenses. Dates and numbers use
        these explicit settings; values are never rounded.
      </p>
      <div class="settings">
        <label>
          Delimiter<select bind:value={layout.delimiter} disabled={saving}>
            <option value=",">Comma</option><option value=";">Semicolon</option>
          </select>
        </label>
        <label>
          Date format<select bind:value={layout.date_format} disabled={saving}>
            <option>YYYY-MM-DD</option><option>DD.MM.YYYY</option><option>DD/MM/YYYY</option>
          </select>
        </label>
        <label>
          Decimal separator<select bind:value={layout.decimal} disabled={saving}>
            <option value=".">Dot</option><option value=",">Comma</option>
          </select>
        </label>
        <label>
          Grouping separator<select bind:value={layout.grouping} disabled={saving}>
            <option value="">None</option><option value=".">Dot</option><option value=",">
              Comma
            </option><option value=" ">Space</option>
          </select>
        </label>
      </div>
    </details>
    <section class="preview-panel" aria-label="Interpreted entry preview">
      <h3>Entry preview</h3>
      <p class="muted">
        Check dates, signs and descriptions before confirming. Individual row problems can be left
        out or descriptions supplied in review.
      </p>
      {#if reading}<p role="status">Reading sample…</p>
      {:else if rows.length}<ul class="entry-preview">
          {#each rows as row (row.record)}<li>
              <div>
                <strong>{row.description || 'Add description in review'}</strong><span
                  class="muted"
                >
                  {row.date}
                </span>{#if row.error || row.description_error}<span class="error">
                    {row.error || row.description_error}
                  </span>{/if}
              </div>
              <span>{row.amount_cents ? money(row.amount_cents, true) : 'Check amount'}</span>
            </li>{/each}
        </ul>
      {:else}<p>Map all three fields to see interpreted entries.</p>{/if}
    </section>
  </div>
  <form class="profile-save" onsubmit={save}>
    <label>Profile name<input bind:value={name} required maxlength="80" disabled={saving} /></label>
    <p class="muted">
      Saving a profile adds no entries. Saved profiles and sources remain if you cancel the import.
    </p>
    <div class="actions">
      <button type="button" class="button quiet" disabled={saving} onclick={onback}>
        Back to file & profile
      </button><button
        class="button primary"
        disabled={saving || reading || !complete || !rows.length}
      >
        {saving ? 'Saving…' : saved ? 'Save changes & review' : 'Save profile & review'}
      </button>
    </div>
  </form>
</section>

<style>
  section {
    min-width: 0;
  }
  .mapping-workspace {
    --date-fill: light-dark(#e6eff8, #24374b);
    --date-ink: light-dark(#285578, #b5d7f4);
    --amount-fill: light-dark(#f8edd4, #40341d);
    --amount-ink: light-dark(#795315, #efd091);
    --description-fill: light-dark(#f0e7f3, #392b40);
    --description-ink: light-dark(#674176, #dfbce9);
    display: grid;
    gap: 1.25rem;
  }
  .mapping-workspace > h3,
  .mapping-workspace > p {
    margin: 0;
  }
  [data-field='date'] {
    --field-fill: var(--date-fill);
    --field-ink: var(--date-ink);
  }
  [data-field='amount'] {
    --field-fill: var(--amount-fill);
    --field-ink: var(--amount-ink);
  }
  [data-field='description'] {
    --field-fill: var(--description-fill);
    --field-ink: var(--description-ink);
  }
  .fields,
  .settings {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 160px), 1fr));
    gap: 1rem;
  }
  .field {
    display: grid;
    text-align: left;
    overflow-wrap: anywhere;
    background: var(--field-fill);
    color: var(--field-ink);
    border: 1px solid color-mix(in srgb, var(--field-ink) 35%, transparent);
    padding: 1rem;
    gap: 0.75rem;
    min-height: 110px;
    align-content: start;
  }
  .field-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .field-number {
    display: inline-grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 50%;
    background: var(--field-ink);
    color: var(--field-fill);
    font-weight: 700;
  }
  .field-state {
    margin-left: auto;
    font-size: 0.875rem;
  }
  .field-column {
    font-size: 1.1rem;
    font-weight: 500;
  }
  .mapping-prompt {
    padding: 1rem 1.25rem;
    border-left: 4px solid var(--field-ink, var(--accent));
    background: var(--field-fill, var(--soft));
    border-radius: 0.5rem;
  }
  .active {
    outline: 3px solid var(--field-ink);
    outline-offset: 2px;
  }
  .table-scroll {
    overflow-x: auto;
    max-width: 100%;
    border: 1px solid var(--line);
    border-radius: 0.6rem;
    max-height: 360px;
  }
  table {
    border-collapse: collapse;
    min-width: 100%;
  }
  th,
  td {
    text-align: left;
    border-bottom: 1px solid var(--line);
    border-right: 1px solid var(--line);
    padding: 1rem;
    min-width: 180px;
    max-width: 320px;
    overflow-wrap: anywhere;
  }
  th:last-child,
  td:last-child {
    border-right: 0;
  }
  th button {
    display: block;
    width: 100%;
    min-height: 100px;
    text-align: left;
    background: transparent;
    color: inherit;
    border: 0;
    border-radius: 0;
    padding: 1rem;
    font: inherit;
    cursor: pointer;
    outline-offset: -3px;
  }
  .column-action {
    display: block;
    margin-top: 0.5rem;
    color: var(--muted);
    font-size: 0.875rem;
    font-weight: 400;
  }
  th button:not(:disabled):hover .column-action,
  th button:focus-visible .column-action {
    color: var(--hover-ink);
  }
  .table-scroll[data-active='date'] {
    --hover-ink: var(--date-ink);
  }
  .table-scroll[data-active='amount'] {
    --hover-ink: var(--amount-ink);
  }
  .table-scroll[data-active='description'] {
    --hover-ink: var(--description-ink);
  }
  th {
    padding: 0;
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--soft);
  }
  th[data-field],
  td[data-field] {
    background: var(--field-fill);
  }
  th[data-field] {
    color: var(--field-ink);
    box-shadow: inset 0 4px var(--field-ink);
  }
  th.column-hover,
  td.column-hover {
    background: color-mix(in srgb, var(--hover-ink) 14%, var(--field-fill, var(--paper)));
  }
  th.column-hover {
    box-shadow: inset 0 -3px var(--hover-ink);
  }
  th button:disabled {
    opacity: 1;
    cursor: default;
  }
  .interpretation-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    align-items: start;
    gap: 1.5rem;
  }
  .reading-panel,
  .preview-panel {
    padding: 1.25rem;
    border: 1px solid var(--line);
    border-radius: 0.8rem;
    background: var(--paper);
  }
  .preview-panel h3 {
    margin-top: 0;
  }
  .profile-save {
    padding-top: 1.5rem;
    border-top: 1px solid var(--line);
  }
  .profile-save input {
    max-width: 32rem;
  }
  @media (max-width: 760px) {
    .interpretation-grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  @media (max-width: 520px) {
    .fields {
      grid-template-columns: minmax(0, 1fr);
    }
    .field {
      min-height: 88px;
    }
    .entry-preview li {
      flex-wrap: wrap;
    }
  }
  .badge {
    display: block;
    font-size: 0.875rem;
    color: var(--field-ink);
  }
  details {
    margin: 0;
  }
  summary {
    min-height: 44px;
    cursor: pointer;
  }
  .entry-preview {
    list-style: none;
    padding: 0;
  }
  .entry-preview li {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--line);
    overflow-wrap: anywhere;
    font-variant-numeric: tabular-nums;
  }
  .entry-preview li div {
    min-width: 0;
    display: grid;
  }
  .entry-preview li > span {
    flex-shrink: 0;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
</style>
