export interface User {
  id: string;
  username: string;
}
export interface Session {
  user: User | null;
  csrf_token: string;
}
export interface Account {
  id: string;
  name: string;
  opening_date: string;
  opening_cents: string;
  balance_cents: string;
  latest_transaction_date: string | null;
}
export interface Transaction {
  id: string;
  account_id: string;
  account_name: string;
  date: string;
  description: string;
  amount_cents: string;
}
export interface Summary {
  month: string;
  income_cents: string;
  spending_cents: string;
  net_cents: string;
  transaction_count: number;
}

let csrf = '';
export function ledgerQuery(month: string, accountId: string): string {
  const query = new URLSearchParams({ month });
  if (accountId) query.set('account_id', accountId);
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
