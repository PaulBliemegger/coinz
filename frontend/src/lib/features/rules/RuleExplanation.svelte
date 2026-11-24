<script lang="ts">
  import { operatorNames, type RuleSuggestions } from './types.ts';
  import type { ClassificationCatalog } from '#lib/features/classification/types.ts';
  let {
    plan,
    catalog,
    confirmed = false,
  }: { plan: RuleSuggestions; catalog: ClassificationCatalog; confirmed?: boolean } = $props();
</script>

{#if plan.matched_rules.length}
  <details class="rule-explanation" data-entry-controls>
    <summary>
      {plan.matched_rules.length === 1
        ? `Rule · ${plan.matched_rules[0].name}`
        : `${plan.matched_rules.length} matching rules`}{confirmed ? ' · Your category kept' : ''}
    </summary>
    {#each plan.matched_rules as rule (rule.id)}<p>
        {rule.name}: description {operatorNames[rule.operator]} “{rule.pattern}”{rule.category_id
          ? ` → ${catalog.categories.find((c) => c.id === rule.category_id)?.name ?? 'Category unavailable'}`
          : ''}.
      </p>{/each}
  </details>
  {#if plan.conflict && !confirmed}<p class="rule-conflict" role="status">
      Rules suggest different categories: {plan.category_candidates
        .map((id) => catalog.categories.find((c) => c.id === id)?.name ?? 'Unavailable')
        .join(' or ')}. Choose a category, or explicitly choose Uncategorized.
    </p>{/if}
{/if}
