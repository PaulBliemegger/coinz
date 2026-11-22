<script lang="ts">
  let {
    label,
    selected,
    disabled = false,
    onselect,
  }: {
    label: string;
    selected: boolean;
    disabled?: boolean;
    onselect: (selected: boolean) => void;
  } = $props();
</script>

<label class="selection-control">
  <span class="sr-only">{label}</span><input
    type="checkbox"
    checked={selected}
    {disabled}
    onchange={(event) => onselect(event.currentTarget.checked)}
  />
  <span class="selection-circle" aria-hidden="true">{selected ? '−' : ''}</span>
</label>

<style>
  .selection-control {
    position: relative;
    min-height: 44px;
    width: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: start;
  }
  input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    min-height: 0;
    opacity: 0;
    margin: 0;
    cursor: inherit;
  }
  .selection-circle {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    border: 1px solid var(--muted);
    background: var(--paper);
    color: var(--ink);
    font-size: 1.25rem;
    line-height: 1;
    pointer-events: none;
  }
  input:checked + .selection-circle {
    background: var(--accent);
    color: light-dark(#fffaf4, #302016);
    border-color: var(--accent);
  }
  input:focus-visible + .selection-circle {
    outline: 3px solid var(--accent);
    outline-offset: 3px;
  }
  .selection-control:has(input:disabled) {
    opacity: 0.6;
  }
</style>
