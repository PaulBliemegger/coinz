import { expect, test } from '@playwright/test';

test('manual entry without setup, optional sources, exact totals, editing, and logout', async ({
  page,
}, testInfo) => {
  const browserErrors: string[] = [];
  page.on('pageerror', (error) => browserErrors.push(error.message));
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${testInfo.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Where did it go?' })).toBeVisible();
  await expect(page.getByRole('button', { name: '+ Add transaction', exact: true })).toBeEnabled();
  await expect(page.getByRole('button', { name: 'Accounts', exact: true })).toHaveCount(0);

  for (const entry of [
    { kind: 'Income', description: 'Salary', amount: '2000', date: '2026-11-02' },
    { kind: 'Expense', description: 'A very precise lunch', amount: '12,34', date: '2026-11-03' },
  ]) {
    await page.getByRole('button', { name: '+ Add transaction', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: 'Add a transaction' });
    await dialog.getByRole('radio', { name: entry.kind, exact: true }).check();
    await dialog.getByLabel('Amount').fill(entry.amount);
    await dialog.getByLabel('Description').fill(entry.description);
    await dialog.getByLabel('Date', { exact: true }).fill(entry.date);
    await dialog.getByRole('button', { name: 'Record transaction' }).click();
    await expect(dialog).not.toBeVisible();
  }
  await expect(page.getByTestId('spending')).toHaveText('€12.34');
  await expect(page.getByTestId('income')).toHaveText('€2,000.00');
  await page.getByRole('button', { name: 'Edit A very precise lunch' }).click();
  const editDialog = page.getByRole('dialog', { name: 'Edit transaction' });
  await editDialog.getByLabel('Amount').fill('13.34');
  await editDialog.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  await page.getByLabel('Month', { exact: true }).fill('2026-12');
  await expect(page.getByTestId('spending')).toHaveText('€0.00');
  await expect(
    page.getByRole('heading', { name: 'Nothing recorded for this period.' }),
  ).toBeVisible();
  await page.getByLabel('Month', { exact: true }).fill('2026-11');
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  await expect(page.locator('body')).toContainText('not a live bank connection');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await page.screenshot({
    path: testInfo.outputPath(`coinz-${testInfo.project.name}.png`),
    fullPage: true,
  });
  // Optional provenance can be added to an existing manual entry without changing totals.
  await page.getByRole('button', { name: 'Edit A very precise lunch' }).click();
  await page.getByRole('button', { name: '+ Add source', exact: true }).click();
  const sourceDialog = page.getByRole('dialog', { name: 'Add a source' });
  await sourceDialog.getByLabel('Source name').fill('N26');
  await sourceDialog.getByRole('button', { name: 'Create source' }).click();
  await expect(sourceDialog).not.toBeVisible();
  await expect(editDialog.getByLabel('Source (optional)', { exact: true })).not.toHaveValue('');
  await editDialog.getByRole('button', { name: 'Save changes' }).click();
  await page.getByLabel('Source', { exact: true }).selectOption({ label: 'N26' });
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  await expect(page.getByTestId('income')).toHaveText('€0.00');
  await page.getByLabel('Source', { exact: true }).selectOption('');
  await expect(page.getByTestId('income')).toHaveText('€2,000.00');
  await page.getByRole('button', { name: 'Edit A very precise lunch' }).click();
  await editDialog.getByLabel('Source (optional)', { exact: true }).selectOption('');
  await editDialog.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  expect(browserErrors).toEqual([]);
  await page.getByRole('button', { name: 'Log out' }).click();
  await expect(page.getByRole('button', { name: 'Log in', exact: true })).toBeVisible();
});

test('wrong credentials keep the ledger private', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill('desktop-user');
  await page.getByLabel('Password', { exact: true }).fill('wrong-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Username or password is incorrect');
  await expect(page.getByRole('heading', { name: 'Where did it go?' })).not.toBeVisible();
});
