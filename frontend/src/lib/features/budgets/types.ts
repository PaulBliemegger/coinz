import type { ClassificationItem } from '../classification/types.ts';
export interface BudgetCategory {
  category: ClassificationItem;
  budget_cents: string | null;
  spending_cents: string;
  remaining_cents: string | null;
  year_spending_cents: string;
  year_budget_cents: string;
  year_budgeted_spending_cents: string;
  budgeted_months: number;
}
export interface ExpectedIncome {
  id: string;
  name: string;
  planned_cents: string;
  complete: boolean;
  transaction_ids: string[];
  received_cents: string;
  pending_cents: string;
  invalid_receipt_count: number;
  unrestored_receipt_count: number;
}
export interface BudgetPlan {
  month: string;
  as_of: string;
  revision: number;
  savings_cents: string;
  categories: BudgetCategory[];
  income: ExpectedIncome[];
  receipts: { id: string; date: string; description: string; amount_cents: string }[];
  recorded_income_cents: string;
  spending_cents: string;
  unbudgeted_spending_cents: string;
  budget_total_cents: string;
  planned_income_cents: string;
  pending_income_cents: string;
  unallocated_cents: string;
  from_recorded_cents: string;
  including_pending_cents: string;
}
export interface IncomeDraft {
  key: string;
  id: string | null;
  name: string;
  amount: string;
  complete: boolean;
  transaction_ids: string[];
}
