import { test } from 'node:test';
import assert from 'node:assert/strict';
import { filterChoices } from '../src/lib/features/ledger/filter-choices.ts';
import type { ClassificationItem } from '../src/lib/features/classification/types.ts';

test('filter choices retain selected archived items, search names, and prioritize selections', () => {
  const items: ClassificationItem[] = [
    { id: 'local', name: 'Local shop', archived: false, color: 'sage' },
    { id: 'old', name: 'Old holiday', archived: true, color: 'blue' },
    { id: 'food', name: 'Eating out', archived: false, color: 'clay' },
    { id: 'archived', name: 'Archived shop', archived: true, color: 'sand' },
  ];
  const ids = (choices: ClassificationItem[]) => choices.map((item) => item.id);
  assert.deepEqual(ids(filterChoices(items, ['old', 'local'], '', false)), [
    'local',
    'old',
    'food',
  ]);
  assert.deepEqual(ids(filterChoices(items, ['old'], '  HOLIDAY  ', false)), ['old']);
  assert.deepEqual(ids(filterChoices(items, [], 'shop', false)), ['local']);
  assert.deepEqual(ids(filterChoices(items, [], 'shop', true)), ['archived', 'local']);
  assert.deepEqual(filterChoices(items, [], 'no match', true), []);
  assert.deepEqual(ids(items), ['local', 'old', 'food', 'archived']);
});
