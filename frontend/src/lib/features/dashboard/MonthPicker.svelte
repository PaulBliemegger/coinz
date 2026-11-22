<script lang="ts">
  import { tick } from 'svelte';
  import { localDate, monthLabel } from '#lib/money.ts';
  import { adjacentMonth } from '#lib/features/ledger/period.ts';

  let { value, onchange }: { value: string; onchange: (month: string) => void } = $props();
  let open = $state(false);
  let year = $state<number | undefined>(undefined);
  let root: HTMLDivElement;
  let trigger: HTMLButtonElement;
  let grid = $state<HTMLDivElement>();
  const months = Array.from({ length: 12 }, (_, index) => String(index + 1).padStart(2, '0'));
  let validYear = $derived(
    year !== undefined && Number.isInteger(year) && year >= 1 && year <= 9999,
  );
  let previous = $derived(adjacentMonth(value, -1));
  let next = $derived(adjacentMonth(value, 1));

  function close(restoreFocus = false) {
    open = false;
    if (restoreFocus) trigger.focus();
  }
  async function toggle() {
    if (open) return close();
    year = Number(value.slice(0, 4));
    open = true;
    await tick();
    if (open) grid?.querySelector<HTMLButtonElement>('[aria-pressed="true"]')?.focus();
  }
  function choose(month: string) {
    onchange(month);
    close(true);
  }
  function browse(direction: -1 | 1) {
    if (validYear && year !== undefined) year += direction;
  }
  function monthKey(event: KeyboardEvent, index: number) {
    const offsets: Record<string, number> = {
      ArrowLeft: -1,
      ArrowRight: 1,
      ArrowUp: -3,
      ArrowDown: 3,
    };
    const target =
      event.key === 'Home' ? 0 : event.key === 'End' ? 11 : index + (offsets[event.key] ?? 0);
    if (!(event.key in offsets) && event.key !== 'Home' && event.key !== 'End') return;
    event.preventDefault();
    grid?.querySelectorAll<HTMLButtonElement>('button')[Math.max(0, Math.min(11, target))]?.focus();
  }
</script>

<svelte:window
  onpointerdown={(event) => {
    if (open && event.target instanceof Node && !root.contains(event.target)) close();
  }}
  onkeydown={(event) => {
    if (open && event.key === 'Escape') {
      event.preventDefault();
      close(true);
    }
  }}
/>

<div
  class="overview-picker"
  bind:this={root}
  onfocusout={(event) => {
    if (open && event.relatedTarget instanceof Node && !root.contains(event.relatedTarget)) close();
  }}
>
  <div class="month-navigation" role="group" aria-label="Overview month">
    <button
      class="button arrow"
      aria-label="Previous month"
      disabled={!previous}
      onclick={() => {
        if (previous) {
          close();
          onchange(previous);
        }
      }}
    >
      ←
    </button>
    <button
      class="button month-trigger"
      bind:this={trigger}
      aria-expanded={open}
      aria-controls="overview-month-options"
      onclick={toggle}
    >
      {monthLabel(value)} <span class="chevron" aria-hidden="true">⌄</span>
    </button>
    <button
      class="button arrow"
      aria-label="Next month"
      disabled={!next}
      onclick={() => {
        if (next) {
          close();
          onchange(next);
        }
      }}
    >
      →
    </button>
  </div>
  {#if open}
    <div
      class="month-popover"
      id="overview-month-options"
      role="group"
      aria-label="Choose overview month"
    >
      <div class="year-navigation">
        <button
          class="button arrow"
          aria-label="Previous year"
          disabled={!validYear || year === 1}
          onclick={() => browse(-1)}
        >
          ←
        </button>
        <label class="year-label">
          Year<input type="number" min="1" max="9999" step="1" required bind:value={year} />
        </label>
        <button
          class="button arrow"
          aria-label="Next year"
          disabled={!validYear || year === 9999}
          onclick={() => browse(1)}
        >
          →
        </button>
      </div>
      {#if !validYear}<p class="year-error" role="status">Choose a year from 1 to 9999.</p>{/if}
      <div class="month-grid" bind:this={grid}>
        {#each months as month, index (month)}
          {@const option = `${String(year ?? 1).padStart(4, '0')}-${month}`}
          <button
            class="button month-option"
            disabled={!validYear}
            aria-label={validYear ? monthLabel(option) : monthLabel(`2000-${month}`).split(' ')[0]}
            aria-pressed={validYear && option === value}
            onclick={() => choose(option)}
            onkeydown={(event) => monthKey(event, index)}
          >
            {monthLabel(`2000-${month}`).slice(0, 3)}
          </button>
        {/each}
      </div>
      <div class="picker-footer">
        <button class="button quiet" onclick={() => choose(localDate().slice(0, 7))}>
          This month
        </button>
        <button class="button quiet" onclick={() => close(true)}>Close</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .overview-picker {
    position: relative;
    width: fit-content;
    max-width: 100%;
    margin: 0 auto 1.5rem;
    z-index: 2;
  }
  .month-navigation,
  .year-navigation,
  .picker-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }
  .month-navigation {
    gap: 0.25rem;
  }
  .arrow {
    width: 44px;
    padding: 0;
    border-color: transparent;
    background: transparent;
    flex-shrink: 0;
    font-size: 1.2rem;
  }
  .arrow:disabled {
    cursor: default;
    opacity: 0.35;
  }
  .month-trigger {
    border-radius: 999px;
    gap: 0.75rem;
    font-size: 1rem;
    padding-inline: 0.75rem;
  }
  .chevron {
    color: var(--muted);
  }
  .month-popover {
    position: absolute;
    top: calc(100% + 0.6rem);
    left: 50%;
    transform: translateX(-50%);
    width: min(300px, calc(100vw - 2rem));
    padding: 0.75rem;
    border: 1px solid var(--line);
    border-radius: 1.1rem;
    background: var(--paper);
    box-shadow: 0 12px 30px #0002;
  }
  .year-label {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0;
    color: var(--muted);
    font-size: 0.875rem;
    min-width: 0;
  }
  .year-label input {
    width: 7rem;
    min-height: 44px;
    padding: 0.25rem 0.5rem;
    margin: 0;
    text-align: center;
    color: var(--ink);
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 0.6rem;
  }
  .month-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.35rem;
    margin: 0.75rem 0;
  }
  .month-option {
    border-color: transparent;
    border-radius: 999px;
    background: transparent;
  }
  .month-option[aria-pressed='true'] {
    background: var(--accent);
    color: light-dark(#fffaf4, #302016);
  }
  .month-option[aria-pressed='true']:hover {
    background: var(--accent-hover);
  }
  .month-option:disabled {
    cursor: default;
  }
  .picker-footer {
    border-top: 1px solid var(--line);
    padding-top: 0.5rem;
  }
  .year-error {
    color: var(--accent);
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }
</style>
