import { test } from 'node:test';
import assert from 'node:assert/strict';
import { budgetEntriesPath } from '../src/lib/features/budgets/entry-link.ts';
test('budget drill-down follows the cutoff for future, current and historical months', () => {
  assert.equal(budgetEntriesPath('2026-12', 'synthetic-category', '2026-11-24'), null);
  const current = new URL(
    budgetEntriesPath('2026-11', 'synthetic-category', '2026-11-24')!,
    'http://coinz.test',
  );
  assert.equal(current.searchParams.get('end'), '2026-11-24');
  assert.equal(current.searchParams.get('category_ids'), 'synthetic-category');
  const past = new URL(
    budgetEntriesPath('2024-02', 'synthetic-category', '2026-11-24')!,
    'http://coinz.test',
  );
  assert.equal(past.searchParams.get('end'), '2024-02-29');
});
