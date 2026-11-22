export function adjacentMonth(value: string, direction: -1 | 1): string | null {
  if (!/^\d{4}-\d{2}$/.test(value)) return null;
  const [year, month] = value.split('-').map(Number);
  if (year < 1 || year > 9999 || month < 1 || month > 12) return null;
  const index = (year - 1) * 12 + month - 1 + direction;
  if (index < 0 || index >= 9999 * 12) return null;
  return `${String(Math.floor(index / 12) + 1).padStart(4, '0')}-${String((index % 12) + 1).padStart(2, '0')}`;
}
