import { expect, test, type Page } from '@playwright/test';

async function chooseMonth(page: Page, year: string, month: string) {
  await page
    .getByRole('group', { name: 'Overview month', exact: true })
    .locator('.month-trigger')
    .click();
  await page.getByRole('spinbutton', { name: 'Year', exact: true }).fill(year);
  await page
    .getByRole('group', { name: 'Choose overview month', exact: true })
    .getByRole('button', { name: `${month} ${year}`, exact: true })
    .click();
}

test('monthly category spending links to its records, including archived and uncategorized', async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${info.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
  const token = (await (await page.request.get('/api/session')).json()).csrf_token;
  async function post(path: string, data: unknown) {
    const response = await page.request.post(`/api${path}`, {
      headers: { 'X-CSRF-Token': token },
      data,
    });
    expect(response.ok(), `${path}: ${response.status()} ${await response.text()}`).toBeTruthy();
    return response.status() === 204 ? undefined : response.json();
  }
  // The native test server owns only fresh temporary synthetic data.
  for (;;) {
    const existing = await (await page.request.get('/api/entries?page_size=100')).json();
    if (!existing.entries.length) break;
    await post('/transactions/bulk-delete', {
      ids: existing.entries.map((entry: { id: string }) => entry.id),
    });
  }
  const ids: string[] = [];
  for (const [index, amount] of ['60', '50', '40', '30', '20', '10', '5'].entries()) {
    const category = await post('/classification', {
      kind: 'category',
      name: `Insights fixture ${index}`,
      color: 'sage',
    });
    const entry = await post('/transactions', {
      date: '2026-11-30',
      description: `Insights spending ${index}`,
      kind: 'expense',
      amount,
    });
    ids.push(category.id);
    await post('/transactions/classify', {
      ids: [entry.id],
      action: 'set_category',
      category_id: category.id,
    });
  }
  await post('/transactions', {
    date: '2026-11-01',
    description: 'Insights unclassified',
    kind: 'expense',
    amount: '2.50',
  });
  const income = await post('/transactions', {
    date: '2026-11-01',
    description: 'Insights income',
    kind: 'income',
    amount: '200',
  });
  await post('/transactions/classify', {
    ids: [income.id],
    action: 'set_category',
    category_id: ids[0],
  });
  const archived = await page.request.put(`/api/classification/${ids[0]}`, {
    headers: { 'X-CSRF-Token': token },
    data: { name: 'Insights fixture 0', color: 'sage', archived: true },
  });
  expect(archived.ok()).toBeTruthy();
  await post('/transactions', {
    date: '2026-12-01',
    description: 'Insights following month',
    kind: 'expense',
    amount: '999',
  });
  await page.reload();
  await chooseMonth(page, '2026', 'November');
  const panel = page.getByRole('region', { name: 'Spending by category', exact: true });
  await expect(page.getByTestId('spending')).toHaveText('€217.50');
  await expect(panel.getByRole('listitem')).toHaveCount(6);
  await expect(panel.getByRole('link', { name: /Insights fixture 0/ })).toContainText('27.6%');
  await expect(panel.getByRole('link', { name: /Insights fixture 0/ })).toContainText('Archived');
  await expect(panel.getByRole('link', { name: /1 uncategorized spending entry/ })).toBeVisible();
  await panel.getByRole('button', { name: 'Show all 8 categories', exact: true }).click();
  await expect(panel.getByRole('listitem')).toHaveCount(8);
  await page.getByRole('link', { name: 'Explore insights' }).click();
  await expect(page).toHaveURL(/\/insights\?month=2026-11$/);
  await expect(
    page.getByRole('heading', { name: 'Follow the patterns.', exact: true }),
  ).toBeVisible();
  await expect(page.getByRole('link', { name: 'Insights', exact: true })).toHaveAttribute(
    'aria-current',
    'page',
  );
  await expect(panel.getByRole('listitem')).toHaveCount(8);
  await expect(panel.getByRole('button', { name: /Show all/ })).toHaveCount(0);
  await page.screenshot({ path: info.outputPath('category-spending.png'), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await panel.getByRole('link', { name: /Insights fixture 0/ }).click();
  await expect(page.getByRole('radio', { name: 'Expenses', exact: true })).toBeChecked();
  await expect(page.getByLabel('From', { exact: true })).toHaveValue('2026-11-01');
  await expect(page.getByLabel('Through', { exact: true })).toHaveValue('2026-11-30');
  await expect(page.getByTestId('spending')).toHaveText('€60.00');
  await expect(page.getByTestId('income')).toHaveText('€0.00');
  await page.getByRole('button', { name: /^Categories/ }).click();
  await expect(page.getByLabel('Insights fixture 0 · Archived', { exact: true })).toBeChecked();
  await page.getByRole('radio', { name: 'All', exact: true }).check();
  await expect(page.getByTestId('income')).toHaveText('€200.00');
  await page.getByRole('link', { name: 'Home', exact: true }).click();
  await chooseMonth(page, '2026', 'November');
  await panel.getByRole('link', { name: /1 uncategorized spending entry/ }).click();
  await expect(page.getByTestId('spending')).toHaveText('€2.50');
  await page.getByRole('button', { name: /^Categories/ }).click();
  await expect(page.getByLabel('Uncategorized', { exact: true })).toBeChecked();
  await page.getByRole('link', { name: 'Home', exact: true }).click();
  await chooseMonth(page, '2025', 'January');
  await expect(panel).toContainText('No spending recorded for January 2025 yet.');
  await expect(panel.getByRole('link')).toHaveCount(0);
  await page.goto('/insights?month=2026-11');
  const trends = page.getByRole('region', { name: 'Monthly trends', exact: true });
  const flow = trends.getByRole('region', { name: 'Income and expenses over time', exact: true });
  const categoryTrend = trends.getByRole('region', {
    name: 'Category spending over time',
    exact: true,
  });
  await expect(flow).toBeVisible();
  await trends.getByLabel('Trend category', { exact: true }).selectOption(ids[0]);
  await expect(categoryTrend).toContainText('Insights fixture 0');
  await page.route(
    '**/api/insights/trends*',
    (route) => route.fulfill({ status: 503, json: { error: 'Synthetic trend read failure' } }),
    { times: 1 },
  );
  await trends.getByLabel('Trend category', { exact: true }).selectOption(ids[1]);
  await expect(trends).toContainText('Synthetic trend read failure');
  await expect(flow).toHaveCount(0);
  await trends.getByRole('button', { name: 'Retry trends', exact: true }).click();
  await expect(flow.getByRole('link', { name: /^Income, / })).toHaveCount(12);
  await trends.getByLabel('Trend category', { exact: true }).selectOption(ids[0]);
  await expect(categoryTrend).toContainText('Insights fixture 0');
  await categoryTrend.getByText('Exact figures: Insights fixture 0', { exact: true }).click();
  await expect(categoryTrend.getByRole('row').filter({ hasText: 'November 2026' })).toContainText(
    '€60.00',
  );
  await page.screenshot({ path: info.outputPath('insights-trends.png'), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await flow.getByRole('link', { name: /^Income, November 2026:/ }).click();
  await expect(page.getByRole('radio', { name: 'Income', exact: true })).toBeChecked();
  await expect(page.getByTestId('income')).toHaveText('€200.00');
  await expect(page.getByTestId('spending')).toHaveText('€0.00');
  await page.goto('/insights?month=2026-11');
  await trends.getByLabel('Trend category', { exact: true }).selectOption(ids[0]);
  await categoryTrend
    .getByRole('link', { name: /^Expenses, Insights fixture 0, November 2026:/ })
    .click();
  await expect(page.getByRole('radio', { name: 'Expenses', exact: true })).toBeChecked();
  await expect(page.getByTestId('spending')).toHaveText('€60.00');
  await expect(page.getByLabel('From', { exact: true })).toHaveValue('2026-11-01');
  expect(errors).toEqual([]);
});
