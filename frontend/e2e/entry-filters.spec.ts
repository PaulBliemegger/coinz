import { expect, test } from '@playwright/test';

test('compact entry filters search, combine, clear and retain period and description scope', async ({
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
  const headers = { 'X-CSRF-Token': token };
  async function post(path: string, data: unknown) {
    const response = await page.request.post(`/api${path}`, { headers, data });
    expect(response.ok()).toBeTruthy();
    return response.status() === 204 ? undefined : response.json();
  }
  const groceries = await post('/classification', {
    kind: 'category',
    name: 'Filter groceries',
    color: 'sage',
  });
  const archived = await post('/classification', {
    kind: 'category',
    name: 'Filter archived',
    color: 'sand',
  });
  const recurring = await post('/classification', {
    kind: 'label',
    name: 'Filter recurring',
    color: 'blue',
  });
  const home = await post('/classification', { kind: 'label', name: 'Filter home', color: 'clay' });
  for (const [amount, kind, category, labels] of [
    ['10', 'expense', groceries.id, [recurring.id, home.id]],
    ['20', 'expense', groceries.id, [recurring.id]],
    ['50', 'income', groceries.id, [recurring.id]],
    ['5', 'expense', null, []],
    ['8', 'expense', archived.id, [home.id]],
  ] as [string, string, string | null, string[]][]) {
    const entry = await post('/transactions', {
      date: '2026-11-15',
      description: 'Filter fixture entry',
      kind,
      amount,
    });
    if (category)
      await post('/transactions/classify', {
        ids: [entry.id],
        action: 'set_category',
        category_id: category,
      });
    if (labels.length)
      await post('/transactions/classify', {
        ids: [entry.id],
        action: 'add_labels',
        label_ids: labels,
      });
  }
  expect(
    (
      await page.request.put(`/api/classification/${archived.id}`, {
        headers,
        data: { name: 'Filter archived', color: 'sand', archived: true },
      })
    ).ok(),
  ).toBeTruthy();
  await post('/transactions', {
    date: '2026-10-31',
    description: 'Filter fixture prior month',
    kind: 'expense',
    amount: '999',
  });
  await page.getByRole('link', { name: 'Entries', exact: true }).click();
  await page.getByRole('radio', { name: 'Custom', exact: true }).check();
  await page.getByLabel('From', { exact: true }).fill('2026-11-01');
  await page.getByLabel('Through', { exact: true }).fill('2026-11-30');
  await page.getByLabel('Search descriptions').fill('Filter fixture');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  const filters = page.getByRole('region', { name: 'Classification filters' });
  await filters.getByRole('radio', { name: 'Expenses', exact: true }).check();
  await expect(page.getByTestId('spending')).toHaveText('€43.00');
  await expect(page.getByTestId('income')).toHaveText('€0.00');
  const categories = filters.getByRole('button', { name: /^Categories/ });
  const labels = filters.getByRole('button', { name: /^Labels/ });
  await categories.click();
  const categoryPicker = page.getByRole('region', { name: 'Filter categories', exact: true });
  await expect(categoryPicker.getByLabel('Search categories')).toBeFocused();
  await categoryPicker.getByLabel('Search categories').fill('groCERies');
  await categoryPicker.getByLabel('Filter groceries', { exact: true }).check();
  await expect(page.getByTestId('spending')).toHaveText('€30.00');
  await page.keyboard.press('Escape');
  await expect(categoryPicker).not.toBeVisible();
  await expect(categories).toBeFocused();
  await filters
    .getByRole('button', { name: 'Remove category filter Filter groceries', exact: true })
    .click();
  await expect(categories).toBeFocused();
  await expect(page.getByTestId('spending')).toHaveText('€43.00');
  await labels.click();
  const labelPicker = page.getByRole('region', { name: 'Filter labels', exact: true });
  await labelPicker.getByLabel('Search labels').fill('Filter');
  await labelPicker.getByLabel('Filter recurring', { exact: true }).check();
  await labelPicker.getByLabel('Filter home', { exact: true }).check();
  await expect(page.getByTestId('spending')).toHaveText('€38.00');
  await labelPicker.getByRole('radio', { name: 'All', exact: true }).check();
  await expect(page.getByTestId('spending')).toHaveText('€10.00');
  await labelPicker.getByRole('button', { name: 'Close labels filter' }).click();
  await expect(filters.getByRole('group', { name: 'Active filters' })).toContainText(
    'Match all labels',
  );
  await filters
    .getByRole('button', { name: 'Remove label filter Filter home', exact: true })
    .click();
  await expect(page.getByTestId('spending')).toHaveText('€30.00');
  await labels.click();
  await labelPicker.getByLabel('Unlabeled', { exact: true }).check();
  await expect(labelPicker.getByLabel('Filter recurring', { exact: true })).not.toBeChecked();
  await expect(page.getByTestId('spending')).toHaveText('€5.00');
  await page.keyboard.press('Escape');
  await filters.getByRole('button', { name: 'Clear filters', exact: true }).click();
  await expect(filters.getByRole('radio', { name: 'All', exact: true })).toBeChecked();
  await expect(page.getByTestId('spending')).toHaveText('€43.00');
  await expect(page.getByTestId('income')).toHaveText('€50.00');
  await expect(page.getByLabel('From', { exact: true })).toHaveValue('2026-11-01');
  await expect(page.getByLabel('Through', { exact: true })).toHaveValue('2026-11-30');
  await expect(page.getByLabel('Search descriptions')).toHaveValue('Filter fixture');
  await expect(filters.getByRole('group', { name: 'Active filters' })).toHaveCount(0);
  await categories.click();
  await categoryPicker.getByLabel('Search categories').fill('archived');
  await expect(
    categoryPicker.getByLabel('Filter archived · Archived', { exact: true }),
  ).toHaveCount(0);
  await categoryPicker.getByLabel('Include archived', { exact: true }).check();
  await categoryPicker.getByLabel('Filter archived · Archived', { exact: true }).check();
  await categoryPicker.getByLabel('Include archived', { exact: true }).uncheck();
  await expect(
    categoryPicker.getByLabel('Filter archived · Archived', { exact: true }),
  ).toBeChecked();
  await expect(page.getByTestId('spending')).toHaveText('€8.00');
  await page.screenshot({ path: info.outputPath('entry-filter-picker.png'), fullPage: true });
  const box = await categoryPicker.boundingBox();
  expect(box).not.toBeNull();
  expect(box!.x).toBeGreaterThanOrEqual(0);
  expect(box!.x + box!.width).toBeLessThanOrEqual(page.viewportSize()!.width);
  await page.getByLabel('Search descriptions').click();
  await expect(categoryPicker).not.toBeVisible();
  await filters.getByRole('button', { name: 'Clear filters', exact: true }).click();
  await filters.getByRole('radio', { name: 'Income', exact: true }).check();
  await expect(page.getByTestId('income')).toHaveText('€50.00');
  await expect(page.getByTestId('spending')).toHaveText('€0.00');
  await filters.getByRole('button', { name: 'Remove entry type filter' }).click();
  await expect(page.getByTestId('spending')).toHaveText('€43.00');
  expect(errors).toEqual([]);
});
