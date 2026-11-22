import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseAppearance } from '../src/lib/appearance.ts';
import { adjacentMonth } from '../src/lib/features/ledger/period.ts';

test('appearance follows system unless an explicit valid override is stored', () => {
  assert.equal(parseAppearance('dark'), 'dark');
  assert.equal(parseAppearance('light'), 'light');
  for (const value of [null, undefined, '', 'invalid', 'system', {}, 1])
    assert.equal(parseAppearance(value), 'system');
});
test('month navigation crosses years without wrapping the supported calendar', () => {
  assert.equal(adjacentMonth('2026-12', 1), '2027-01');
  assert.equal(adjacentMonth('2026-01', -1), '2025-12');
  assert.equal(adjacentMonth('0001-01', -1), null);
  assert.equal(adjacentMonth('9999-12', 1), null);
  assert.equal(adjacentMonth('0001-01', 1), '0001-02');
  for (const value of ['', '2026-13', '0000-01', '2026-1'])
    assert.equal(adjacentMonth(value, 1), null);
});
