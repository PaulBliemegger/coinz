import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spendingEntriesPath } from '../src/lib/features/dashboard/category-spending.ts';

test('spending drill-down keeps category and exact calendar-month boundaries', () => {
  for (const [month, end] of [
    ['2026-11', '2026-11-30'],
    ['2024-02', '2024-02-29'],
    ['2100-02', '2100-02-28'],
    ['2000-02', '2000-02-29'],
    ['0004-02', '0004-02-29'],
    ['0001-01', '0001-01-31'],
    ['9999-12', '9999-12-31'],
  ]) {
    const url = new URL(spendingEntriesPath(month, 'synthetic-category'), 'http://coinz.test');
    assert.equal(url.searchParams.get('start'), `${month}-01`);
    assert.equal(url.searchParams.get('end'), end);
    assert.equal(url.searchParams.get('entry_type'), 'spending');
    assert.equal(url.searchParams.get('category_ids'), 'synthetic-category');
    assert.equal(url.searchParams.has('uncategorized'), false);
  }
  const uncategorized = new URL(spendingEntriesPath('2026-11', null), 'http://coinz.test');
  assert.equal(uncategorized.searchParams.get('uncategorized'), 'true');
  assert.equal(uncategorized.searchParams.has('category_ids'), false);
});
