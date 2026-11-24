import { spendingEntriesPath } from '../dashboard/category-spending.ts';

export function budgetEntriesPath(
  month: string,
  categoryId: string,
  asOf: string,
): `/entries?${string}` | null {
  if (asOf < `${month}-01`) return null;
  const params = new URLSearchParams(spendingEntriesPath(month, categoryId).split('?')[1]);
  const end = params.get('end')!;
  if (asOf < end) params.set('end', asOf);
  return `/entries?${params}`;
}
