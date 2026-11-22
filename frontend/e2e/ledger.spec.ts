import { expect, test, type Page } from '@playwright/test';

async function login(page: Page, username: string) {
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(username);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
}
async function csrf(page: Page) {
  return (await (await page.request.get('/api/session')).json()).csrf_token as string;
}
async function clean(page: Page) {
  const token = await csrf(page);
  for (;;) {
    const data = await (await page.request.get('/api/entries?page_size=100')).json();
    if (!data.entries.length) break;
    const response = await page.request.post('/api/transactions/bulk-delete', {
      headers: { 'X-CSRF-Token': token },
      data: { ids: data.entries.map((entry: { id: string }) => entry.id) },
    });
    expect(response.ok()).toBeTruthy();
  }
  await page.reload();
}
async function add(
  page: Page,
  description: string,
  amount: string,
  date: string,
  kind = 'Expense',
) {
  await page.getByRole('button', { name: '+ Add transaction', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Add a transaction' });
  await dialog.getByRole('radio', { name: kind, exact: true }).check();
  await dialog.getByLabel('Amount').fill(amount);
  await dialog.getByLabel('Description').fill(description);
  await dialog.getByLabel('Date', { exact: true }).fill(date);
  await dialog.getByRole('button', { name: 'Record transaction', exact: true }).click();
  await expect(dialog).not.toBeVisible();
}
async function importFile(page: Page, name: string, csv: string, sourceName: string) {
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Import CSV' });
  await dialog.getByRole('button', { name: '+ New source', exact: true }).click();
  await dialog.getByLabel('New source name').fill(sourceName);
  await dialog.getByRole('button', { name: 'Create import source', exact: true }).click();
  await dialog
    .getByLabel('CSV file')
    .setInputFiles({ name, mimeType: 'text/csv', buffer: Buffer.from(csv) });
  await dialog.getByRole('button', { name: 'Preview CSV', exact: true }).click();
  return dialog;
}

test('dashboard, entry workspace, exact totals, editing, filters, theme and logout', async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(e.message));
  await login(page, `${info.project.name}-user`);
  await clean(page);
  await page
    .getByRole('group', { name: 'Overview month', exact: true })
    .locator('.month-trigger')
    .click();
  await page.getByRole('spinbutton', { name: 'Year', exact: true }).fill('2026');
  await page
    .getByRole('group', { name: 'Choose overview month', exact: true })
    .getByRole('button', { name: 'November 2026', exact: true })
    .click();
  await add(page, 'Salary', '2000', '2026-11-02', 'Income');
  await add(page, 'A very precise lunch', '12,34', '2026-11-03');
  await expect(page.getByTestId('spending')).toHaveText('€12.34');
  await expect(page.getByTestId('income')).toHaveText('€2,000.00');
  await expect(page.getByRole('heading', { name: 'Latest entries' })).toBeVisible();
  await expect(page.locator('.latest-entries li')).toHaveCount(2);
  await expect(page.locator('.latest-entries li').first()).toHaveCSS(
    'border-bottom-style',
    'solid',
  );
  await page.screenshot({ path: info.outputPath('dashboard-light.png'), fullPage: true });
  await page.getByRole('link', { name: 'View all' }).click();
  await expect(page.getByRole('radio', { name: 'Custom', exact: true })).toBeChecked();
  await page
    .getByRole('button', { name: 'Change amount: A very precise lunch', exact: true })
    .click();
  await page.getByLabel('Amount', { exact: true }).fill('13.34');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByTestId('spending')).toHaveText('€13.34');
  await page.getByLabel('Search descriptions').fill('lunch');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page.getByTestId('income')).toHaveText('€0.00');
  await page.getByLabel('From', { exact: true }).fill('2026-12-01');
  await expect(
    page.getByRole('heading', { name: 'Nothing recorded for this period.' }),
  ).toBeVisible();
  await page.getByLabel('From', { exact: true }).fill('2026-11-01');
  await page.locator('.user-menu > summary').click();
  await page.getByLabel('Appearance', { exact: true }).selectOption('dark');
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByLabel('Always show selection controls').check();
  await page.getByRole('button', { name: 'Close settings', exact: true }).click();
  await expect(page.locator('html')).toHaveAttribute('data-appearance', 'dark');
  await page.getByRole('checkbox', { name: 'Select displayed entries', exact: true }).check();
  await page.getByRole('button', { name: 'Delete selected entries', exact: true }).click();
  const bin = await page
    .getByRole('button', { name: 'Delete selected entries', exact: true })
    .boundingBox();
  const confirm = await page
    .getByRole('button', { name: 'Delete 1 entry', exact: true })
    .boundingBox();
  expect(bin && confirm && Math.abs(bin.y - confirm.y) < 100).toBeTruthy();
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBeTruthy();
  await page.screenshot({ path: info.outputPath('entries-dark.png'), fullPage: true });
  await page.getByRole('button', { name: 'Delete selected entries', exact: true }).click();
  await page.getByRole('button', { name: 'Delete 1 entry', exact: true }).click();
  await expect(page.getByTestId('spending')).toHaveText('€0.00');
  await page.locator('.user-menu > summary').click();
  await page.getByRole('button', { name: 'Log out', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Log in', exact: true })).toBeVisible();
  expect(errors).toEqual([]);
});

test('overview month picker browses years, supports keys and respects calendar boundaries', async ({
  page,
}, info) => {
  await login(page, `${info.project.name}-user`);
  const navigation = page.getByRole('group', { name: 'Overview month', exact: true });
  const trigger = navigation.locator('.month-trigger');
  const chooser = page.getByRole('group', { name: 'Choose overview month', exact: true });
  await trigger.click();
  await page.getByRole('spinbutton', { name: 'Year', exact: true }).fill('2026');
  await chooser.getByRole('button', { name: 'November 2026', exact: true }).click();
  await expect(trigger).toBeFocused();
  await expect(trigger).toContainText('November 2026');
  await trigger.click();
  await expect(chooser.getByRole('button', { name: 'November 2026', exact: true })).toBeFocused();
  await page.getByRole('button', { name: 'Next year', exact: true }).click();
  await expect(trigger).toContainText('November 2026');
  await expect(page.getByRole('heading', { name: '2026 spending', exact: true })).toBeVisible();
  const year = page.getByRole('spinbutton', { name: 'Year', exact: true });
  await year.fill('0');
  await expect(chooser.getByRole('button', { name: 'January', exact: true })).toBeDisabled();
  await expect(chooser.getByRole('status')).toHaveText('Choose a year from 1 to 9999.');
  await year.fill('2026');
  await chooser.getByRole('button', { name: 'November 2026', exact: true }).focus();
  await page.keyboard.press('ArrowRight');
  await expect(chooser.getByRole('button', { name: 'December 2026', exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await navigation.getByRole('button', { name: 'Next month', exact: true }).click();
  await expect(trigger).toContainText('January 2027');
  await expect(navigation.getByRole('button', { name: 'Next month', exact: true })).toBeFocused();
  await navigation.getByRole('button', { name: 'Previous month', exact: true }).click();
  await expect(trigger).toContainText('December 2026');
  await trigger.click();
  await page.keyboard.press('Escape');
  await expect(chooser).not.toBeVisible();
  await expect(trigger).toBeFocused();
  await trigger.click();
  await page.getByRole('heading', { name: 'The bigger picture.' }).click();
  await expect(chooser).not.toBeVisible();
  await trigger.click();
  await year.fill('1');
  await expect(chooser.getByRole('button', { name: 'Previous year', exact: true })).toBeDisabled();
  await chooser.getByRole('button', { name: 'January 1', exact: true }).click();
  await expect(
    navigation.getByRole('button', { name: 'Previous month', exact: true }),
  ).toBeDisabled();
  await trigger.click();
  await year.fill('9999');
  await expect(chooser.getByRole('button', { name: 'Next year', exact: true })).toBeDisabled();
  await chooser.getByRole('button', { name: 'December 9999', exact: true }).click();
  await expect(navigation.getByRole('button', { name: 'Next month', exact: true })).toBeDisabled();
  await trigger.click();
  await chooser.getByRole('button', { name: 'This month', exact: true }).click();
  const current = await page.evaluate(() =>
    new Intl.DateTimeFormat('en', { month: 'long', year: 'numeric' }).format(new Date()),
  );
  await expect(trigger).toContainText(current);
  await trigger.click();
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBeTruthy();
  await page.screenshot({ path: info.outputPath('month-picker.png'), fullPage: true });
  await chooser.getByRole('button', { name: 'Close', exact: true }).click();
  await expect(trigger).toBeFocused();
});

test('CSV subset, repeat preview, history removal and protected edits across pages', async ({
  page,
}, info) => {
  await login(page, `${info.project.name}-user`);
  await clean(page);
  const csv =
    'date,description,amount,external_id\n' +
    Array.from(
      { length: 32 },
      (_, i) =>
        `2026-${i % 2 ? '11' : '10'}-05,Demo purchase ${String(i).padStart(2, '0')},-1,id-${i}\n`,
    ).join('');
  const filename = `synthetic-batch-${info.project.name}.csv`;
  let dialog = await importFile(page, filename, csv, `History ${info.project.name}`);
  await expect(dialog.getByRole('heading', { name: 'Review 32 entries' })).toBeVisible();
  const importRow = dialog.locator('.review-row').filter({
    has: dialog.getByRole('checkbox', { name: 'Include Demo purchase 00', exact: true }),
  });
  const include = importRow.getByRole('checkbox', {
    name: 'Include Demo purchase 00',
    exact: true,
  });
  await expect(importRow).toHaveClass(/selected/);
  await expect(importRow.locator('.selection-circle')).toHaveText('−');
  await importRow.locator('summary').click();
  await expect(include).toBeChecked();
  await importRow.getByRole('heading', { name: 'Demo purchase 00', exact: true }).click();
  await expect(include).not.toBeChecked();
  await expect(importRow).not.toHaveClass(/\bselected\b/);
  await include.focus();
  await page.keyboard.press('Space');
  await expect(include).toBeChecked();
  await expect(importRow).toHaveClass(/selected/);

  await dialog.getByRole('button', { name: 'Import 32 entries', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await page.getByRole('link', { name: 'View all' }).click();
  await expect(page.getByText('32 matching', { exact: true })).toBeVisible();
  await expect(page.locator('.entry-row')).toHaveCount(25);
  await expect(page.getByTestId('spending')).toHaveText('€32.00');
  await page.getByRole('checkbox', { name: 'Select displayed entries', exact: true }).check();
  await page.getByRole('button', { name: 'Next', exact: true }).click();
  await expect(page.locator('.entry-row')).toHaveCount(7);
  await expect(page.getByRole('button', { name: 'Delete selected entries' })).toHaveCount(0);
  await page.getByRole('radio', { name: 'Year', exact: true }).check();
  await page.getByRole('spinbutton', { name: 'Year', exact: true }).fill('2026');
  await expect(page.getByText('32 matching', { exact: true })).toBeVisible();
  await page
    .getByRole('button', { name: 'Change description: Demo purchase 31', exact: true })
    .click();
  await page.getByLabel('Description', { exact: true }).fill('Edited purchase');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await page
    .getByRole('button', { name: 'Change description: Demo purchase 29', exact: true })
    .click();
  await page.getByLabel('Description', { exact: true }).fill('Other edited purchase');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await page.getByRole('link', { name: 'Imports', exact: true }).click();
  await page.getByRole('button', { name: `Review ${filename}`, exact: true }).click();
  await expect(page.getByRole('button', { name: 'Remove 30 entries', exact: true })).toBeVisible();
  await expect(
    page.getByRole('list', { name: 'Entries included in removal' }).locator('li'),
  ).toHaveCount(5);
  await page
    .locator('details')
    .filter({ hasText: 'Edited entries · 2' })
    .locator('summary')
    .click();
  const firstEdited = page.getByRole('checkbox', {
    name: 'Include Edited purchase in removal',
    exact: true,
  });
  const otherEdited = page.getByRole('checkbox', {
    name: 'Include Other edited purchase in removal',
    exact: true,
  });
  await expect(firstEdited).not.toBeChecked();
  await expect(otherEdited).not.toBeChecked();
  const editedRow = page
    .getByRole('list', { name: 'Edited entries', exact: true })
    .locator('li')
    .filter({ has: firstEdited });
  await editedRow.getByText('Edited purchase', { exact: true }).click();
  await expect(firstEdited).toBeChecked();
  await expect(editedRow).toHaveClass(/selected/);
  await expect(editedRow.locator('.selection-circle')).toHaveText('−');
  await expect(page.getByRole('button', { name: 'Remove 31 entries', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Remove 31 entries', exact: true }).click();
  await otherEdited.check();
  await expect(
    page.getByRole('button', { name: 'Confirm removal', exact: true }),
  ).not.toBeVisible();
  await otherEdited.uncheck();
  await page.getByRole('button', { name: 'Remove 31 entries', exact: true }).click();
  await page.screenshot({ path: info.outputPath('import-removal.png'), fullPage: true });
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBeTruthy();
  await page.getByRole('button', { name: 'Confirm removal', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('31 imported entries removed');
  await expect(page.getByRole('button', { name: 'Remove 0 entries', exact: true })).toBeDisabled();
  await page
    .locator('details')
    .filter({ hasText: 'Edited entries · 1' })
    .locator('summary')
    .click();
  await otherEdited.check();
  await page.getByRole('button', { name: 'Remove 1 entries', exact: true }).click();
  await page.getByRole('button', { name: 'Confirm removal', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('1 imported entries removed');
  // Deletion releases all evidence so the file can be imported again.
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  dialog = page.getByRole('dialog', { name: 'Import CSV' });
  await dialog.getByLabel('Import source').selectOption({ label: `History ${info.project.name}` });
  await dialog
    .getByLabel('CSV file')
    .setInputFiles({ name: filename, mimeType: 'text/csv', buffer: Buffer.from(csv) });
  await dialog.getByRole('button', { name: 'Preview CSV', exact: true }).click();
  await expect(
    dialog.getByRole('button', { name: 'Import 32 entries', exact: true }),
  ).toBeEnabled();
  await dialog.getByRole('checkbox', { name: 'Select new entries', exact: true }).uncheck();
  await dialog.getByRole('checkbox', { name: 'Include Demo purchase 00', exact: true }).check();
  await dialog.getByRole('button', { name: 'Import 1 entries', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  dialog = page.getByRole('dialog', { name: 'Import CSV' });
  await dialog.getByLabel('Import source').selectOption({ label: `History ${info.project.name}` });
  await dialog
    .getByLabel('CSV file')
    .setInputFiles({ name: filename, mimeType: 'text/csv', buffer: Buffer.from(csv) });
  await dialog.getByRole('button', { name: 'Preview CSV', exact: true }).click();
  await expect(
    dialog.getByRole('button', { name: 'Import 31 entries', exact: true }),
  ).toBeEnabled();
  await expect(dialog.getByText('1 already imported · excluded', { exact: true })).toBeVisible();
  await dialog.getByText('1 already imported · excluded', { exact: true }).click();
  const knownRow = dialog.locator('.review-row').filter({
    has: dialog.getByRole('checkbox', { name: 'Include Demo purchase 00', exact: true }),
  });
  await expect(knownRow.getByRole('checkbox')).toBeDisabled();
  await knownRow.getByRole('heading', { name: 'Demo purchase 00', exact: true }).click();
  await expect(knownRow).not.toHaveClass(/\bselected\b/);
  await expect(
    dialog.getByRole('button', { name: 'Import 31 entries', exact: true }),
  ).toBeEnabled();

  await dialog.getByRole('button', { name: 'Close import', exact: true }).click();
});

test('incorrect credentials keep private screens hidden', async ({ page }) => {
  await page.goto('/entries');
  await page.getByLabel('Username', { exact: true }).fill('desktop-user');
  await page.getByLabel('Password', { exact: true }).fill('wrong-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Username or password is incorrect');
  await expect(page.getByRole('navigation', { name: 'Main navigation' })).toHaveCount(0);
});

test('row backgrounds and round selectors toggle independently of inline editing', async ({
  page,
}, info) => {
  await login(page, `${info.project.name}-user`);
  await clean(page);
  await add(page, 'Selection example', '4.25', '2026-11-03');
  await page.getByRole('link', { name: 'View all' }).click();
  const row = page.locator('.entry-row').filter({
    has: page.getByRole('button', { name: 'Change description: Selection example', exact: true }),
  });
  const checkbox = row.getByRole('checkbox', { name: 'Select Selection example', exact: true });
  const background = async () => {
    const box = await row.boundingBox();
    if (!box) throw new Error('Missing synthetic entry row');
    await row.click({ position: { x: box.width - 4, y: box.height - 4 } });
  };
  await background();
  await expect(checkbox).toBeChecked();
  await expect(row).toHaveClass(/selected/);
  expect(await row.evaluate((node) => getComputedStyle(node, '::after').content)).toBe('none');
  await expect(row.locator('.selection-circle')).toHaveText('−');
  await row
    .getByRole('button', { name: 'Change description: Selection example', exact: true })
    .click();
  await row.getByLabel('Description', { exact: true }).fill('Unsaved draft');
  await expect(checkbox).toBeChecked();
  await checkbox.uncheck();
  await expect(row.getByLabel('Description', { exact: true })).toHaveValue('Unsaved draft');
  await expect(row).not.toHaveClass(/\bselected\b/);
  await row.getByRole('button', { name: 'Cancel', exact: true }).click();
  await checkbox.focus();
  await page.keyboard.press('Space');
  await expect(checkbox).toBeChecked();
  await page.keyboard.press('Space');
  await expect(checkbox).not.toBeChecked();
  await background();
  await background();
  await background();
  await expect(checkbox).toBeChecked();
  await page.getByRole('button', { name: 'Clear selection', exact: true }).click();
  await expect(checkbox).not.toBeChecked();
  await expect
    .poll(() =>
      row.locator('.selection-highlight').evaluate((node) => getComputedStyle(node).opacity),
    )
    .toBe('0');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await background();
  await expect(checkbox).toBeChecked();
  expect(
    await row
      .locator('.selection-highlight')
      .evaluate(
        (node) =>
          node
            .getAnimations({ subtree: true })
            .filter((animation) => animation.playState === 'running').length,
      ),
  ).toBe(0);
  await expect(
    row.getByRole('button', { name: 'Change description: Selection example', exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBeTruthy();
  await page.screenshot({ path: info.outputPath('row-selection.png'), fullPage: true });
});

test('overlapping imports retain one creator and disclose cross-import removals', async ({
  page,
}, info) => {
  await login(page, `${info.project.name}-user`);
  await clean(page);
  const headers = { 'X-CSRF-Token': await csrf(page) };
  const created = await page.request.post('/api/sources', {
    headers,
    data: { name: 'Overlap browser fixture' },
  });
  expect(created.ok()).toBeTruthy();
  const source = (await created.json()).id;
  const csv =
    'date,description,amount,external_id\n2026-10-01,Overlap one,-1,overlap-one\n2026-11-01,Overlap two,-2,overlap-two\n';
  const first = `creator-${info.project.name}.csv`;
  const second = `later-${info.project.name}.csv`;
  for (const [filename, content] of [
    [first, csv],
    [second, csv + '2026-11-02,Later addition,-3,later-three\n'],
  ]) {
    const response = await page.request.post('/api/imports/commit', {
      headers,
      data: { source_id: source, filename, csv: content },
    });
    expect(response.ok()).toBeTruthy();
  }
  await page.getByRole('link', { name: 'Imports', exact: true }).click();
  await page.getByRole('button', { name: `Review ${first}`, exact: true }).click();
  await expect(page.getByRole('button', { name: 'Remove 2 entries', exact: true })).toBeEnabled();
  await expect(
    page.getByText(
      '2 selected entries also appear in other imports. They will be deleted from your records.',
      { exact: true },
    ),
  ).toBeVisible();
  await expect(page.getByText('Shared entries', { exact: false })).toHaveCount(0);
  await page.getByRole('button', { name: 'Remove 2 entries', exact: true }).click();
  await page.screenshot({ path: info.outputPath('overlap-removal.png'), fullPage: true });
  await page.getByRole('button', { name: 'Confirm removal', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('2 imported entries removed');
  await page.getByRole('button', { name: '← All imports', exact: true }).click();
  await page.getByRole('button', { name: `Review ${second}`, exact: true }).click();
  await expect(page.getByRole('button', { name: 'Remove 1 entries', exact: true })).toBeEnabled();
  await expect(page.getByRole('list', { name: 'Entries included in removal' })).toContainText(
    'Later addition',
  );
  await page.getByRole('link', { name: 'Entries', exact: true }).click();
  await page.getByRole('radio', { name: 'Custom', exact: true }).check();
  await expect(page.getByText('1 matching', { exact: true })).toBeVisible();
  await expect(page.getByTestId('spending')).toHaveText('€3.00');
});
