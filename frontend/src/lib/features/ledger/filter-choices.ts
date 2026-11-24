import type { ClassificationItem } from '../classification/types.ts';

export function filterChoices(
  items: ClassificationItem[],
  selectedIds: string[],
  search: string,
  includeArchived: boolean,
): ClassificationItem[] {
  const query = search.trim().toLowerCase();
  const selected = new Set(selectedIds);
  return items
    .filter(
      (item) =>
        (includeArchived || !item.archived || selected.has(item.id)) &&
        item.name.toLowerCase().includes(query),
    )
    .sort(
      (a, b) =>
        Number(selected.has(b.id)) - Number(selected.has(a.id)) || a.name.localeCompare(b.name),
    );
}
