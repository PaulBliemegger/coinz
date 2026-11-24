import { test } from 'node:test';
import assert from 'node:assert/strict';
import { barPercent, monthStatus, chartScale } from '../src/lib/features/insights/trend-display.ts';
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

test('euro ticks use small readable steps and keep large chart axes bounded', () => {
  assert.deepEqual(chartScale(19000n), { top: 20000n, ticks: [0n, 5000n, 10000n, 15000n, 20000n] });
  const scale = chartScale(250000n);
  assert.equal(scale.top, 250000n);
  assert.ok(scale.ticks.length <= 6);
  assert.ok(chartScale(9223372036854775807n).top >= 9223372036854775807n);
});
