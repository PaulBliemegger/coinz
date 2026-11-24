import { test } from 'node:test';
import assert from 'node:assert/strict';
import { barPercent, monthStatus } from '../src/lib/features/insights/trend-display.ts';
import { monthEntriesPath } from '../src/lib/features/dashboard/category-spending.ts';

test('chart scaling keeps large cents exact until converting a bounded visual ratio', () => {
  assert.equal(barPercent('9007199254740993', 9007199254740993n), 100);
  assert.equal(barPercent('0', 0n), 0);
  assert.equal(barPercent('1', 3n), 33.33);
  assert.equal(barPercent('0', 100n), 0);
});
test('month markers follow the local calendar, not whether data happens to exist', () => {
  assert.equal(monthStatus('2026-11', '2026-11-24'), 'In progress');
  assert.equal(monthStatus('2026-12', '2026-11-24'), 'Future');
  assert.equal(monthStatus('2026-10', '2026-11-24'), '');
});
test('income bars open income entries with correct leap-month scope and no category filter', () => {
  const url = new URL(monthEntriesPath('2024-02', 'income'), 'http://coinz.test');
  assert.equal(url.searchParams.get('entry_type'), 'income');
  assert.equal(url.searchParams.get('end'), '2024-02-29');
  assert.equal(url.searchParams.has('category_ids'), false);
  assert.equal(url.searchParams.has('uncategorized'), false);
});
