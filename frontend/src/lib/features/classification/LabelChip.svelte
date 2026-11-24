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

<span class="label-chip" data-color={item.color}>
  <span>{item.name}</span>
  {#if onremove}<button
      type="button"
      class="label-remove"
      aria-label={`Remove label ${item.name}`}
      {disabled}
      onclick={onremove}
    >
      ×
    </button>{/if}
</span>
