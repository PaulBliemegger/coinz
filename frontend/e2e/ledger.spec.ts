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
  await page
    .getByRole('button', { name: 'Change amount: A very precise lunch', exact: true })
    .click();
  await page.getByLabel('Amount', { exact: true }).fill('13.34');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
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
  await page.getByRole('button', { name: '+ Add transaction', exact: true }).click();
  await page.getByRole('button', { name: '+ Add source', exact: true }).click();
  const sourceDialog = page.getByRole('dialog', { name: 'Add a source' });
  await sourceDialog.getByLabel('Source name').fill('N26');
  await sourceDialog.getByRole('button', { name: 'Create source' }).click();
  await expect(sourceDialog).not.toBeVisible();
  await page.getByRole('button', { name: 'Close transaction', exact: true }).click();
  await page
    .getByRole('button', { name: 'Change source: A very precise lunch', exact: true })
    .click();
  await page.getByLabel('Source', { exact: true }).last().selectOption({ label: 'N26' });
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await page.getByLabel('Source', { exact: true }).selectOption({ label: 'N26' });
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  await expect(page.getByTestId('income')).toHaveText('€0.00');
  await page.getByLabel('Source', { exact: true }).selectOption('');
  await expect(page.getByTestId('income')).toHaveText('€2,000.00');
  await page
    .getByRole('button', { name: 'Change source: A very precise lunch', exact: true })
    .click();
  await page.getByLabel('Source', { exact: true }).last().selectOption('');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  expect(browserErrors).toEqual([]);
  await page.locator('.user-menu > summary').click();
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

test('CSV preview, cancellation, repeat import, and overlap', async ({ page }, testInfo) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${testInfo.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  let dialog = page.getByRole('dialog', { name: 'Import CSV' });
  await dialog.getByRole('button', { name: '+ New source', exact: true }).click();
  await dialog.getByLabel('New source name').fill('Synthetic CSV stream');
  await dialog.getByRole('button', { name: 'Create import source' }).click();
  await dialog.getByLabel('CSV file').setInputFiles('../examples/imports/demo.csv');
  await dialog.getByRole('button', { name: 'Preview CSV', exact: true }).click();
  await expect(dialog.getByRole('heading', { name: 'Review 16 entries' })).toBeVisible();
  await expect(
    dialog.getByRole('checkbox', { name: 'Select new entries', exact: true }),
  ).toBeChecked();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await page.screenshot({ path: testInfo.outputPath('csv-preview.png'), fullPage: true });
  await dialog.getByRole('button', { name: 'Close import' }).click();
  await expect(dialog).not.toBeVisible();
  // Preview cancellation did not write any imported entries.
  await page.getByLabel('Source', { exact: true }).selectOption({ label: 'Synthetic CSV stream' });
  await expect(page.getByTestId('spending')).toHaveText('€0.00');
  for (const [filename, count, added, known, spending] of [
    ['demo.csv', 16, 16, 0, '€1,063.79'],
    ['demo.csv', 16, 0, 16, '€1,063.79'],
    ['demo-overlap.csv', 4, 1, 3, '€1,079.79'],
  ] as const) {
    await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
    dialog = page.getByRole('dialog', { name: 'Import CSV' });
    await dialog.getByLabel('CSV file').setInputFiles(`../examples/imports/${filename}`);
    await dialog.getByRole('button', { name: 'Preview CSV', exact: true }).click();
    await expect(dialog.getByRole('heading', { name: `Review ${count} entries` })).toBeVisible();
    if (added === 0) {
      await expect(dialog.getByRole('button', { name: /^Import \d+ entries/ })).toBeDisabled();
      await dialog.getByRole('button', { name: 'Close import' }).click();
    } else {
      await dialog.getByRole('button', { name: /^Import \d+ entries/ }).click();
      await expect(page.getByRole('status')).toContainText(
        `${added} added, 0 matched, ${known} already processed`,
      );
    }
    await expect(dialog).not.toBeVisible();
    await expect(page.getByTestId('spending')).toHaveText(spending);
    await expect(page.getByTestId('income')).toHaveText('€2,400.00');
  }
  await page.screenshot({ path: testInfo.outputPath('csv-imported.png'), fullPage: true });
  expect(errors).toEqual([]);
});

test('selection preference, direct editing, and adjacent deletion confirmation', async ({
  page,
}, testInfo) => {
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${testInfo.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await page.getByRole('button', { name: '+ Add transaction', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Add a transaction' });
  await dialog.getByLabel('Amount').fill('5.25');
  await dialog.getByLabel('Description').fill('Bulk example');
  await dialog.getByLabel('Date', { exact: true }).fill('2026-09-01');
  await dialog.getByRole('button', { name: 'Record transaction', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await page.locator('.user-menu > summary').click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByLabel('Always show selection controls').check();
  await page.getByRole('button', { name: 'Close settings' }).click();
  await page.reload();
  await page.getByLabel('Month', { exact: true }).fill('2026-09');
  await page.locator('.user-menu > summary').click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByLabel('Always show selection controls')).toBeChecked();
  await page.getByRole('button', { name: 'Close settings' }).click();
  await page.getByRole('button', { name: 'Change description: Bulk example', exact: true }).click();
  await page.getByLabel('Description', { exact: true }).fill('Cancelled change');
  await page.getByLabel('Description', { exact: true }).press('Escape');
  await expect(
    page.getByRole('button', { name: 'Change description: Bulk example', exact: true }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Change amount: Bulk example', exact: true }).click();
  await page.getByLabel('Amount', { exact: true }).fill('5.26');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByTestId('spending')).toHaveText('€5.26');
  await page.getByLabel('Select Bulk example', { exact: true }).check();
  const bin = page.getByRole('button', { name: 'Delete selected entries', exact: true });
  const before = await bin.boundingBox();
  await bin.click();
  if (testInfo.project.name === 'desktop') {
    const after = await bin.boundingBox();
    const selectLine = await page.locator('.select-all').boundingBox();
    expect(Math.abs(before!.x - after!.x)).toBeLessThan(2);
    expect(Math.abs(before!.y - selectLine!.y)).toBeLessThan(2);
  }
  await expect(page.getByRole('button', { name: 'Delete 1 entry', exact: true })).toBeVisible();
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await bin.press('Escape');
  await expect(page.getByRole('button', { name: 'Delete 1 entry', exact: true })).not.toBeVisible();
  await bin.click();
  await page.screenshot({ path: testInfo.outputPath('bulk-confirmation.png'), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await page.getByRole('button', { name: 'Delete 1 entry', exact: true }).click();
  await expect(page.getByTestId('spending')).toHaveText('€0.00');
  await expect(
    page.getByRole('heading', { name: 'Nothing recorded for this period.' }),
  ).toBeVisible();
});

test('import a selected subset while leaving invalid and unchecked rows for later', async ({
  page,
}, testInfo) => {
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${testInfo.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  const contents = Buffer.from(
    'date,description,amount,external_id\n2026-08-01,Partial first,-7.50,partial-1\n2026-08-02,Partial later,-2.50,partial-2\n2026-02-30,Unreadable example,1.234,bad\n',
  );
  for (const first of [true, false]) {
    await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
    const dialog = page.getByRole('dialog', { name: 'Import CSV' });
    if (first) {
      await dialog.getByRole('button', { name: '+ New source', exact: true }).click();
      await dialog.getByLabel('New source name').fill('Partial synthetic source');
      await dialog.getByRole('button', { name: 'Create import source' }).click();
    }
    await dialog
      .getByLabel('CSV file')
      .setInputFiles({ name: 'synthetic-partial.csv', mimeType: 'text/csv', buffer: contents });
    await dialog.getByRole('button', { name: 'Preview CSV', exact: true }).click();
    await expect(dialog.locator('article').first()).toContainText('Unreadable example');
    if (first)
      await dialog.getByRole('checkbox', { name: 'Include Partial later', exact: true }).uncheck();
    const commit = dialog.getByRole('button', { name: 'Import 1 entries', exact: true });
    await expect(commit).toBeEnabled();
    await commit.click();
    await expect(dialog).not.toBeVisible();
    await expect(page.getByTestId('spending')).toHaveText(first ? '€7.50' : '€10.00');
  }
});

test('system appearance, persisted overrides, and visible month navigation', async ({
  page,
}, testInfo) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${testInfo.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.locator('html')).toHaveCSS('background-color', 'rgb(33, 28, 24)');
  await page.locator('.user-menu > summary').click();
  await expect(page.getByLabel('Appearance', { exact: true })).toHaveValue('system');
  await page.getByLabel('Appearance', { exact: true }).selectOption('light');
  await expect(page.locator('html')).toHaveCSS('background-color', 'rgb(247, 240, 230)');
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-appearance', 'light');
  await page.locator('.user-menu > summary').click();
  await page.getByLabel('Appearance', { exact: true }).selectOption('system');
  await page.emulateMedia({ colorScheme: 'light' });
  await expect(page.locator('html')).toHaveCSS('background-color', 'rgb(247, 240, 230)');
  await page.emulateMedia({ colorScheme: 'dark' });
  await expect(page.locator('html')).toHaveCSS('background-color', 'rgb(33, 28, 24)');
  await page.getByLabel('Appearance', { exact: true }).selectOption('dark');
  await page.locator('.user-menu > summary').press('Escape');
  await page.getByLabel('Month', { exact: true }).fill('2026-12');
  await page.getByRole('button', { name: 'Next month', exact: true }).click();
  await expect(page.getByLabel('Month', { exact: true })).toHaveValue('2027-01');
  await page.getByRole('button', { name: 'Previous month', exact: true }).click();
  await expect(page.getByLabel('Month', { exact: true })).toHaveValue('2026-12');
  await expect(page.locator('summary').filter({ hasText: 'Import details' })).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  );
  await page.screenshot({ path: testInfo.outputPath('compact-dark.png'), fullPage: true });
});
