<script lang="ts">
  import type { ClassificationItem } from './types.ts';
  let {
    item,
    onremove,
    disabled = false,
  }: {
    item: ClassificationItem;
    onremove?: () => void;
    disabled?: boolean;
  } = $props();
</script>

<span class="label-chip" class:removable={onremove !== undefined} data-color={item.color}>
  <span>{item.name}</span>
  {#if onremove}<button
      type="button"
      class="label-remove"
      aria-label={`Remove label ${item.name}`}
      title={`Remove label ${item.name}`}
      {disabled}
      onclick={onremove}
    >
      <span aria-hidden="true">−</span>
    </button>{/if}
</span>
