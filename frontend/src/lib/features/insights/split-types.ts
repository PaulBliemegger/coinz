import type { ClassificationItem } from '../classification/types.ts';
export interface ExpenseSlice {
  category: ClassificationItem | null;
  amount_cents: string;
}
export interface ExpenseSplit {
  month: string;
  months: { month: string; entry_count: number; eligible: boolean }[];
  selected_months: string[];
  current: ExpenseSlice[];
  average: ExpenseSlice[];
  current_total_cents: string;
  average_total_cents: string;
}
