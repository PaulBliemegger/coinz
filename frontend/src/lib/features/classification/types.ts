export const labelColors = ['sage', 'clay', 'sand', 'lavender', 'blue'] as const;
export type LabelColor = (typeof labelColors)[number];
export type ClassificationKind = 'category' | 'label';
export interface ClassificationItem {
  id: string;
  name: string;
  color: LabelColor;
  archived: boolean;
}
export interface ClassificationCatalog {
  categories: ClassificationItem[];
  labels: ClassificationItem[];
  setup_complete: boolean;
}
export type ClassificationChange =
  | { action: 'set_category'; category_id: string | null }
  | { action: 'add_labels' | 'remove_labels'; label_ids: string[] };
export const suggestedCategories = [
  'Housing',
  'Utilities & communication',
  'Groceries',
  'Eating out',
  'Transport',
  'Health',
  'Personal care',
  'Shopping',
  'Leisure',
  'Accommodation',
  'Vacation',
  'Gifts & donations',
  'Salary',
  'Other income',
];
export const suggestedLabels = ['Essential', 'Discretionary', 'Subscription'];
