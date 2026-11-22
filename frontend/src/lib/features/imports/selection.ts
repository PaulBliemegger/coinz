import type { ImportRow } from '#lib/api.ts';

export function importSelection(
  rows: ImportRow[],
  selected: number[],
  choices: Record<number, string>,
) {
  const additions = rows.filter(
    (row) =>
      selected.includes(row.record) &&
      (row.status === 'new' || (row.status === 'review' && choices[row.record] === 'keep')),
  );
  const matches = rows.filter(
    (row) => row.status === 'review' && choices[row.record]?.startsWith('match:'),
  );
  const amounts = additions.map((row) => BigInt(row.amount_cents));
  return {
    additions,
    matches,
    records: [...additions, ...matches].map((row) => row.record),
    decisions: [
      ...additions
        .filter((row) => row.status === 'review')
        .map((row) => ({ record: row.record, action: 'keep' })),
      ...matches.map((row) => ({
        record: row.record,
        action: 'match',
        transaction_id: choices[row.record].slice(6),
      })),
    ],
    income: amounts
      .filter((n) => n > 0n)
      .reduce((a, b) => a + b, 0n)
      .toString(),
    spending: amounts
      .filter((n) => n < 0n)
      .reduce((a, b) => a - b, 0n)
      .toString(),
    excluded:
      rows.filter((row) => row.status !== 'known').length - additions.length - matches.length,
  };
}
