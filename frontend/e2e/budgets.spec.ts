import { expect, test, type Page } from '@playwright/test';
async function chooseMonth(page: Page, month: string) {
  await page
    .getByRole('group', { name: 'Overview month', exact: true })
    .locator('.month-trigger')
    .click();
  await page.getByRole('spinbutton', { name: 'Year', exact: true }).fill('2026');
  await page
    .getByRole('group', { name: 'Choose overview month', exact: true })
    .getByRole('button', { name: `${month} 2026`, exact: true })
    .click();
}
test('budget plans reserve savings, fulfill income once and copy only planning assumptions', async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(e.message));
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${info.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
  const headers = {
    'X-CSRF-Token': (await (await page.request.get('/api/session')).json()).csrf_token,
  };
  async function post(path: string, data: unknown) {
    const r = await page.request.post(`/api${path}`, { headers, data });
    expect(r.ok()).toBeTruthy();
    return r.status() === 204 ? undefined : r.json();
  }
  for (;;) {
    const rows = await (await page.request.get('/api/entries?page_size=100')).json();
    if (!rows.entries.length) break;
    await post('/transactions/bulk-delete', { ids: rows.entries.map((r: { id: string }) => r.id) });
  }
  const food = await post('/classification', {
    kind: 'category',
    name: 'Budget food fixture',
    color: 'sage',
  });
  const zero = await post('/classification', {
    kind: 'category',
    name: 'Budget zero fixture',
    color: 'blue',
  });
  await post('/transactions', {
    date: '2026-11-10',
    kind: 'income',
    amount: '1000',
    description: 'Budget salary fixture',
  });
  for (const [amount, category] of [
    ['100', food.id],
    ['10', zero.id],
    ['50', null],
  ]) {
    const e = await post('/transactions', {
      date: '2026-11-11',
      kind: 'expense',
      amount,
      description: 'Budget expense fixture',
    });
    if (category)
      await post('/transactions/classify', {
        ids: [e.id],
        action: 'set_category',
        category_id: category,
      });
  }
  await page.getByRole('link', { name: 'Budgets', exact: true }).click();
  await page.getByRole('button', { name: 'Edit monthly plan', exact: true }).click();
  await chooseMonth(page, 'November');
  await page.getByLabel('Recorded through', { exact: true }).fill('2026-11-24');
  await page.getByLabel('Recorded through', { exact: true }).press('Tab');
  await expect(page.getByLabel('Budget food fixture budget (€)', { exact: true })).toHaveValue('');
  await page.getByLabel('Reserved for savings (€)', { exact: true }).fill('300');
  await page.getByLabel('Budget food fixture budget (€)', { exact: true }).fill('300');
  await page.getByLabel('Budget zero fixture budget (€)', { exact: true }).fill('0');
  await page.getByRole('button', { name: '+ Expected income', exact: true }).click();
  const income = page.locator('.expected-income-item');
  await income.nth(0).getByLabel('Income name', { exact: true }).fill('Salary');
  await income.nth(0).getByLabel('Expected amount (€)', { exact: true }).fill('2400');
  await income.nth(0).getByText('Match recorded income (0 linked)', { exact: true }).click();
  await income
    .nth(0)
    .getByRole('checkbox', { name: /Budget salary fixture/ })
    .check();
  await page.getByRole('button', { name: '+ Expected income', exact: true }).click();
  await income.nth(1).getByLabel('Income name', { exact: true }).fill('Freelance');
  await income.nth(1).getByLabel('Expected amount (€)', { exact: true }).fill('200');
  await income.nth(1).getByText('Match recorded income (0 linked)', { exact: true }).click();
  await expect(
    income.nth(1).getByRole('checkbox', { name: /Budget salary fixture/ }),
  ).toBeDisabled();
  await page.getByRole('button', { name: 'Save budget', exact: true }).first().click();
  await expect(page.getByRole('status').filter({ hasText: 'Budget saved.' })).toBeVisible();
  await expect(page.locator('.budget-cards').getByText('€540.00', { exact: true })).toBeVisible();
  await expect(page.locator('.budget-cards').getByText('€2,140.00', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Budget zero fixture budget (€)', { exact: true })).toHaveValue(
    '0.00',
  );
  await income.nth(0).getByLabel('Nothing more expected for this item').check();
  await page.getByRole('button', { name: 'Save budget', exact: true }).first().click();
  await expect(page.locator('.budget-cards').getByText('€740.00', { exact: true })).toBeVisible();
  await page.screenshot({ path: info.outputPath('budget-month.png'), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await chooseMonth(page, 'December');
  await page.getByLabel('Copy from month', { exact: true }).fill('2026-11');
  await page.getByRole('button', { name: 'Copy plan', exact: true }).click();
  await expect(income.nth(0).getByLabel('Nothing more expected for this item')).not.toBeChecked();
  await expect(income.nth(0)).toContainText('0 linked');
  await expect(page.getByLabel('Budget food fixture budget (€)', { exact: true })).toHaveValue(
    '300.00',
  );
  await page.getByRole('button', { name: 'Save budget', exact: true }).first().click();
  await expect(page.getByRole('status').filter({ hasText: 'Budget saved.' })).toBeVisible();
  const copied = await (await page.request.get('/api/budgets/2026-12?as_of=2026-11-24')).json();
  expect(copied.income[0].transaction_ids).toEqual([]);
  expect(copied.income[0].received_cents).toBe('0');
  expect(copied.pending_income_cents).toBe('260000');
  expect(errors).toEqual([]);
});
