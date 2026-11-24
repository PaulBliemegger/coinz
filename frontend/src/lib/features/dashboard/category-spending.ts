/** Calendar boundaries stay local strings, including years 0001–0099 and 9999. */
export function spendingEntriesPath(
  month: string,
  categoryId?: string | null,
): `/entries?${string}` {
  return monthEntriesPath(month, 'spending', categoryId);
}

export function monthEntriesPath(
  month: string,
  entryType: 'spending' | 'income',
  categoryId?: string | null,
): `/entries?${string}` {
  const [year, monthNumber] = month.split('-').map(Number);
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][monthNumber - 1];
  const params = new URLSearchParams({
    start: `${month}-01`,
    end: `${month}-${days}`,
    entry_type: entryType,
  });
  if (categoryId === null) params.set('uncategorized', 'true');
  else if (categoryId !== undefined) params.set('category_ids', categoryId);
  return `/entries?${params}`;
}
