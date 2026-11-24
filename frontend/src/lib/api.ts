import type { RuleSuggestions } from '#lib/features/rules/types.ts';
import type { ClassificationItem } from '#lib/features/classification/types.ts';
export interface User {
  id: string;
  username: string;
}
export interface Session {
  user: User | null;
  csrf_token: string;
}
export interface Source {
  id: string;
  name: string;
  latest_transaction_date: string | null;
}
export interface Transaction {
  category: ClassificationItem | null;
  labels: ClassificationItem[];
  id: string;
  source_id: string | null;
  source_name: string | null;
  date: string;
  description: string;
  amount_cents: string;
  origin: 'manual' | 'csv' | 'unknown';
  csv_linked: boolean;
}
export interface Summary {
  month: string;
  income_cents: string;
  spending_cents: string;
  net_cents: string;
  transaction_count: number;
}

export interface ImportRow {
  rule_suggestions?: RuleSuggestions;
  record: number;
  date: string;
  description: string;
  amount_cents: string;
  external_id: string;
  error: string | null;
  description_error?: string | null;
  status: 'new' | 'known' | 'review' | 'conflict' | 'invalid';
  candidates: { id: string; date: string; description: string; amount_cents: string }[];
}
export interface ImportOutcome {
  added: number;
  matched: number;
  skipped: number;
  known: number;
  latest_month: string | null;
  excluded: number;
  income_cents: string;
  spending_cents: string;
}

export interface Provenance {
  source_name: string;
  record_number: number;
  date: string;
  description: string;
  amount_cents: string;
}

let csrf = '';
export function ledgerQuery(month: string, sourceId: string): string {
  const query = new URLSearchParams({ month });
  if (sourceId) query.set('source_id', sourceId);
  return query.toString();
}

export function setCsrf(token: string) {
  csrf = token;
}

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
  ) {
    super(message);
  }
}

export async function api<T>(path: string, method = 'GET', body?: unknown): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api${path}`, {
      method,
      credentials: 'same-origin',
      headers: {
        ...(body !== undefined ? { 'Content-Type': 'application/json' } : {}),
        ...(method !== 'GET' ? { 'X-CSRF-Token': csrf } : {}),
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    throw new ApiError('Could not reach coinz. Check the connection and try again.', 0);
  }
  if (!response.ok) {
    const data = await response.json().catch(() => null);
    throw new ApiError(
      data?.error ?? 'The request could not be completed. Please try again.',
      response.status,
    );
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export interface Totals {
  income_cents: string;
  spending_cents: string;
  net_cents: string;
  transaction_count: number;
}
export interface EntriesPage {
  entries: Transaction[];
  totals: Totals;
  page: number;
  pages: number;
  page_size: number;
}
export interface DashboardData {
  month: string;
  monthly: Totals;
  yearly: Totals;
  months: (Totals & { month: string })[];
  latest: Transaction[];
  latest_date: string | null;
  latest_import: string | null;
  spending_categories: CategorySpending[];
}
export interface CategorySpending {
  category: ClassificationItem | null;
  spending_cents: string;
  entry_count: number;
  share_percent: string;
}
export interface TrendMonth extends Totals {
  month: string;
  category_spending_cents: string;
  category_entry_count: number;
}
export interface InsightTrends {
  category: ClassificationItem | null;
  months: TrendMonth[];
}
export interface ImportBatch {
  id: string;
  filename: string;
  source_name: string;
  created_at: string;
  start_date: string | null;
  end_date: string | null;
  added: number;
  matched: number;
  known: number;
  excluded: number;
  removed: number;
}
export interface ImportEntryPreview {
  id: string;
  date: string;
  description: string;
  amount_cents: string;
  overlapping: boolean;
}
export interface ImportReview {
  batch: ImportBatch;
  removable: number;
  edited: number;
  overlapping: number;
  already_deleted: number;
  removable_entries: ImportEntryPreview[];
  edited_entries: ImportEntryPreview[];
  token: string;
}
export interface ImportHistoryPage {
  batches: ImportBatch[];
  page: number;
  pages: number;
}

export interface ImportLayout {
  date: string;
  amount: string;
  description: string;
  external_id: string | null;
  delimiter: ',' | ';';
  date_format: 'YYYY-MM-DD' | 'DD.MM.YYYY' | 'DD/MM/YYYY';
  decimal: '.' | ',';
  grouping: '' | '.' | ',' | ' ';
}
export interface ImportProfile {
  id: string;
  name: string;
  revision: number;
  layout: ImportLayout;
}
export interface CsvSample {
  headers: string[];
  sample: string[][];
  total: number;
}
