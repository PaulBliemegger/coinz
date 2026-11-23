import { test } from 'node:test';
import assert from 'node:assert/strict';
import { importSelection } from '../src/lib/features/imports/selection.ts';
import type { ImportRow } from '../src/lib/api.ts';

test('selected additions have exact totals, explicit matches, and no permanent skips', () => {
  const row = (record: number, status: ImportRow['status'], cents: string): ImportRow => ({
    record,
    status,
    amount_cents: cents,
    date: '2026-11-01',
    description: 'Synthetic',
    external_id: '',
    error: null,
    candidates: [],
  });
  const rows = [
    row(2, 'new', '9223372036854775807'),
    row(3, 'new', '-234'),
    row(4, 'known', '-123'),
    row(5, 'invalid', ''),
    row(6, 'review', '-345'),
    row(7, 'review', '-456'),
  ];
  const result = importSelection(rows, [2, 3, 4, 5, 7], { 6: 'match:owned-id', 7: 'keep' });
  assert.equal(result.income, '9223372036854775807');
  assert.equal(result.spending, '690');
  assert.deepEqual(result.records, [2, 3, 7, 6]);
  assert.equal(result.excluded, 1);
  assert.deepEqual(result.decisions, [
    { record: 7, action: 'keep' },
    { record: 6, action: 'match', transaction_id: 'owned-id' },
  ]);
  assert.deepEqual(importSelection(rows, [], {}).records, []);
});

test('a missing description blocks additions without blocking an explicit match', () => {
  const row: ImportRow = {
    record: 2,
    status: 'new',
    amount_cents: '-123',
    date: '2026-11-03',
    description: '',
    description_error: 'Add description',
    external_id: '',
    error: null,
    candidates: [],
  };
  assert.deepEqual(importSelection([row], [2], {}).records, []);
  const edited = { ...row, description: 'Lunch with Sam', description_error: null };
  assert.deepEqual(importSelection([edited], [2], {}).records, [2]);
  const duplicate = { ...row, status: 'review' as const };
  assert.deepEqual(importSelection([duplicate], [], { 2: 'match:owned' }).records, [2]);
});
