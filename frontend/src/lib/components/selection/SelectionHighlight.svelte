<script lang="ts">
  import { selectionHighlight } from './selection-highlight.ts';
  let { selected }: { selected: boolean } = $props();
</script>

<span class="selection-highlight" aria-hidden="true" use:selectionHighlight={selected}>
  <span class="selection-flip" data-selection-flip>
    <span class="selection-face selection-front"></span><span class="selection-face selection-back"
    ></span>
  </span>
</span>

<style>
  .selection-highlight {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    pointer-events: none;
    z-index: 0;
    opacity: 0;
    perspective: 650px;
  }
  .selection-flip {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    transform-style: preserve-3d;
    transform: rotateX(180deg);
  }
  .selection-face {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    backface-visibility: hidden;
    -webkit-backface-visibility: hidden;
    background: var(--paper);
  }
  .selection-back {
    background: var(--selection-fill);
    transform: rotateX(180deg);
  }
</style>
