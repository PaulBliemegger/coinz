<script lang="ts">
  import type { Totals } from '#lib/api.ts';
  import { money, monthLabel } from '#lib/money.ts';
  let { month, totals }: { month: string; totals: Totals } = $props();
</script>

<section class="month-summary" aria-label="Monthly summary">
  <div class="spending-total">
    <p class="eyebrow">RECORDED SPENDING</p>
    <p class="hero-amount" data-testid="spending">{money(totals.spending_cents)}</p>
    <p class="muted">{monthLabel(month)} · {totals.transaction_count} entries</p>
  </div>
  <div class="secondary-total">
    <p>Recorded income</p>
    <strong data-testid="income">{money(totals.income_cents)}</strong><span class="muted">
      Same month. Different direction.
    </span>
  </div>
  <div class="secondary-total">
    <p>Income minus spending</p>
    <strong data-testid="net">{money(totals.net_cents, true)}</strong><span class="muted">
      Recorded net, not your bank balance
    </span>
  </div>
</section>
