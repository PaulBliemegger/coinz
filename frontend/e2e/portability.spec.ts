import { expect, test } from '@playwright/test';

test('all-entry CSV download restores classification and optional sources through import review', async ({
  page,
}, info) => {
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${info.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
  const headers = {
    'X-CSRF-Token': (await (await page.request.get('/api/session')).json()).csrf_token,
  };
  async function post(path: string, data: unknown) {
    const response = await page.request.post(`/api${path}`, { headers, data });
    expect(response.ok(), `${path}: ${response.status()} ${await response.text()}`).toBeTruthy();
    return response.status() === 204 ? undefined : response.json();
  }
  for (;;) {
    const existing = await (await page.request.get('/api/entries?page_size=100')).json();
    if (!existing.entries.length) break;
    await post('/transactions/bulk-delete', {
      ids: existing.entries.map((e: { id: string }) => e.id),
    });
  }
  const category = await post('/classification', {
    kind: 'category',
    name: 'Portable dining fixture',
    color: 'sage',
  });
  const label = await post('/classification', {
    kind: 'label',
    name: 'Trip, friends fixture',
    color: 'blue',
  });
  const source = await post('/sources', { name: 'Portable stream fixture' });
  const ids: string[] = [];
  for (const source_id of [source.id, null]) {
    const entry = await post('/transactions', {
      source_id,
      date: '2026-11-03',
      amount: '12.34',
      kind: 'expense',
      description: 'Portable identical coffee',
    });
    ids.push(entry.id);
  }
  await post('/transactions/classify', { ids, action: 'set_category', category_id: category.id });
  await post('/transactions/classify', { ids, action: 'add_labels', label_ids: [label.id] });
  await page.getByRole('link', { name: 'Imports', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Export all entries', exact: true })).toHaveCount(0);
  await page.locator('.user-menu > summary').click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  const settings = page.getByRole('dialog', { name: 'Settings', exact: true });
  await expect(settings.getByRole('region', { name: 'Your data', exact: true })).toBeVisible();
  await settings.screenshot({ path: info.outputPath('settings-export.png') });
  const downloading = page.waitForEvent('download');
  await settings.getByRole('link', { name: 'Export all entries', exact: true }).click();
  const download = await downloading;
  expect(download.suggestedFilename()).toBe('coinz-entries.csv');
  const file = info.outputPath('coinz-entries.csv');
  await download.saveAs(file);
  await settings.getByRole('button', { name: 'Close settings', exact: true }).click();
  await post('/transactions/bulk-delete', { ids });
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Import CSV', exact: true });
  await dialog.getByLabel('CSV file', { exact: true }).setInputFiles(file);
  await expect(dialog).toContainText('Coinz export detected');
  await expect(dialog.getByLabel('Import profile', { exact: true })).toHaveCount(0);
  await dialog.getByRole('button', { name: 'Review entries', exact: true }).click();
  await expect(
    dialog.getByRole('heading', { name: 'Review 2 entries', exact: true }),
  ).toBeVisible();
  await expect(dialog).toContainText('Portable dining fixture');
  await expect(dialog).toContainText('Trip, friends fixture');
  await expect(dialog).toContainText('Source: No source');
  await dialog.screenshot({ path: info.outputPath('coinz-restore-review.png') });
  await dialog.getByRole('button', { name: 'Import 2 entries', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  const entries = (await (await page.request.get('/api/entries')).json()).entries;
  expect(entries).toHaveLength(2);
  expect(
    entries.every(
      (e: { category: { id: string }; labels: { id: string }[] }) =>
        e.category.id === category.id && e.labels.some((l) => l.id === label.id),
    ),
  ).toBe(true);
  expect(entries.filter((e: { source_id: string | null }) => e.source_id === null)).toHaveLength(1);
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  await dialog.getByLabel('CSV file', { exact: true }).setInputFiles(file);
  await expect(dialog).toContainText('Coinz export detected');
  await dialog.getByRole('button', { name: 'Review entries', exact: true }).click();
  await expect(dialog.getByText('2 already imported · excluded', { exact: true })).toBeVisible();
  await expect(
    dialog.getByRole('button', { name: 'Import 0 entries', exact: true }),
  ).toBeDisabled();
  await dialog.getByRole('button', { name: 'Close import', exact: true }).click();
});
