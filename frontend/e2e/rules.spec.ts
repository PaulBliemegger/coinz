import { expect, test } from '@playwright/test';

test('local rules preview, suggest, preserve edits and pause after counted deletion', async ({
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
  for (;;) {
    const data = await (await page.request.get('/api/entries?page_size=100')).json();
    if (!data.entries.length) break;
    expect(
      (
        await page.request.post('/api/transactions/bulk-delete', {
          headers,
          data: { ids: data.entries.map((e: { id: string }) => e.id) },
        })
      ).ok(),
    ).toBeTruthy();
  }
  const category = await page.request.post('/api/classification', {
    headers,
    data: { kind: 'category', name: 'Rule food fixture', color: 'sage' },
  });
  expect(category.ok()).toBeTruthy();
  const categoryId = (await category.json()).id;
  expect(
    (
      await page.request.post('/api/classification', {
        headers,
        data: { kind: 'label', name: 'Rule optional fixture', color: 'blue' },
      })
    ).ok(),
  ).toBeTruthy();
  expect(
    (
      await page.request.post('/api/transactions', {
        headers,
        data: {
          description: 'Coffee fixture',
          date: '2026-11-01',
          kind: 'expense',
          amount: '8.40',
        },
      })
    ).ok(),
  ).toBeTruthy();
  await page.reload();
  await page.getByRole('link', { name: 'Organize', exact: true }).click();
  await page.getByRole('button', { name: 'Rules', exact: true }).click();
  await page.getByRole('button', { name: '+ New rule', exact: true }).click();
  const editor = page.getByRole('region', { name: 'New rule', exact: true });
  await editor.getByLabel('Rule name', { exact: true }).fill('Coffee rule fixture');
  await editor.getByLabel('Match condition', { exact: true }).selectOption('contains_word');
  await editor.getByLabel('Description text to match', { exact: true }).fill('coffee');
  await editor.getByLabel('Set category', { exact: true }).selectOption(categoryId);
  await editor.getByLabel('Rule optional fixture', { exact: true }).check();
  await editor.getByRole('button', { name: 'Preview matching entries', exact: true }).click();
  await expect(editor.getByRole('region', { name: 'Matching entries' })).toContainText(
    '1 matching entries',
  );
  await editor.screenshot({ path: `test-results/rule-editor-${info.project.name}.png` });
  await editor.getByRole('button', { name: 'Save rule', exact: true }).click();
  await expect(editor).not.toBeVisible();
  await page.getByRole('link', { name: 'Home', exact: true }).click();
  await page.getByRole('button', { name: '+ Add transaction', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Add a transaction' });
  await dialog.getByLabel('Description', { exact: true }).fill('Coffee from rule');
  await expect(dialog.getByRole('region', { name: 'Rule suggestions' })).toBeVisible();
  await dialog
    .getByRole('button', { name: 'Use suggestions for untouched fields', exact: true })
    .click();
  await expect(
    dialog.getByRole('button', { name: /Change category.*Rule food fixture/ }),
  ).toBeVisible();
  await dialog.getByRole('button', { name: 'Close transaction', exact: true }).click();
  await page.getByRole('link', { name: 'Entries', exact: true }).click();
  await page.getByRole('radio', { name: 'Custom', exact: true }).check();
  const row = page
    .getByRole('list', { name: 'Recorded entries' })
    .getByRole('listitem')
    .filter({ hasText: 'Coffee fixture' });
  await row.getByRole('button', { name: /Change category.*Uncategorized/ }).click();
  await row.getByRole('button', { name: 'Rule food fixture', exact: true }).click();
  await row.getByRole('button', { name: 'Create rule from this entry…', exact: true }).click();
  const fromEntry = page.getByRole('dialog', { name: 'Create labeling rule' });
  await expect(fromEntry.getByLabel('Description text to match', { exact: true })).toHaveValue(
    'Coffee fixture',
  );
  await expect(fromEntry.getByLabel('Match condition', { exact: true })).toHaveValue('equals');
  await fromEntry.getByRole('button', { name: 'Cancel', exact: true }).click();
  await page.getByRole('link', { name: 'Organize', exact: true }).click();
  await page.getByRole('button', { name: 'Rules', exact: true }).click();
  await page
    .getByRole('button', { name: 'Preview history Coffee rule fixture', exact: true })
    .click();
  const history = page.getByRole('region', { name: 'Apply rule to history' });
  await expect(history).toContainText('1 entries would change');
  await history.getByRole('button', { name: 'Apply to 1 existing entries', exact: true }).click();
  await expect(history).not.toBeVisible();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  await page.getByRole('button', { name: 'Delete Rule optional fixture', exact: true }).click();
  const warning = page.getByRole('alert');
  await expect(warning).toContainText('affects 1 entries');
  await expect(warning).toContainText('1 affected rules');
  await warning.getByRole('button', { name: 'Cancel', exact: true }).click();
  await page.getByRole('button', { name: 'Delete Rule optional fixture', exact: true }).click();
  await page.getByRole('button', { name: 'Delete label', exact: true }).click();
  await page.getByRole('button', { name: 'Rules', exact: true }).click();
  await expect(
    page.getByRole('checkbox', { name: 'Paused Coffee rule fixture', exact: true }),
  ).not.toBeChecked();
  expect((await (await page.request.get('/api/entries')).json()).totals.spending_cents).toBe('840');
  expect(errors).toEqual([]);
});
