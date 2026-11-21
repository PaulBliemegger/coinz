import { expect, test } from '@playwright/test';

test('login, account, exact totals, editing, filtering, and logout', async ({ page }, testInfo) => {
  const browserErrors: string[] = [];
  page.on('pageerror', (error) => browserErrors.push(error.message));
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${testInfo.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Where did it go?' })).toBeVisible();
  await page.getByRole('button', { name: 'Add your first account' }).click();
  const accountDialog = page.getByRole('dialog', { name: 'Add an account' });
  await accountDialog.getByLabel('Account name').fill('N26');
  await accountDialog.getByLabel('Opening date').fill('2026-11-01');
  await accountDialog.getByLabel('Opening balance').fill('1000.00');
  await accountDialog.getByRole('button', { name: 'Create account' }).click();
  await expect(accountDialog).not.toBeVisible();

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
  await expect(page.getByRole('region', { name: 'N26 account' })).toContainText('€2,987.66');
  await page.getByRole('button', { name: 'Edit A very precise lunch' }).click();
  const editDialog = page.getByRole('dialog', { name: 'Edit transaction' });
  await editDialog.getByLabel('Amount').fill('13.34');
  await editDialog.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  await expect(page.getByRole('region', { name: 'N26 account' })).toContainText('€2,986.66');
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
