import { expect, test } from '@playwright/test';

test('settings JSON previews, cancels, downloads and restores into an empty account', async ({
  page,
}, info) => {
  async function login(suffix: string) {
    await page.goto('/');
    await page
      .getByLabel('Username', { exact: true })
      .fill(`${info.project.name}-settings-${suffix}`);
    await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
    await page.getByRole('button', { name: 'Log in', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
    await page.locator('.user-menu > summary').click();
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
  }
  await login('source');
  const csrf = (await (await page.request.get('/api/session')).json()).csrf_token;
  const fixture = {
    format: 'coinz-settings',
    version: 1,
    setup_complete: true,
    classifications: [
      {
        id: '1d7e0230-fbad-4039-b5e6-183e553b4970',
        kind: 'category',
        name: 'Portable dining',
        color: 'clay',
        archived: true,
      },
    ],
    sources: [{ id: '3d7e0230-fbad-4039-b5e6-183e553b4970', name: 'Synthetic stream' }],
    profiles: [],
    rules: [],
    budgets: [
      {
        month: '2026-11',
        savings_cents: '12345',
        categories: [{ category_id: '1d7e0230-fbad-4039-b5e6-183e553b4970', amount_cents: '5000' }],
        income: [],
      },
    ],
  };
  const initialized = await page.request.post('/api/settings/restore/commit', {
    headers: { 'X-CSRF-Token': csrf },
    data: fixture,
  });
  expect(initialized.status()).toBe(204);
  const settings = page.getByRole('dialog', { name: 'Settings', exact: true });
  const downloading = page.waitForEvent('download');
  await settings.getByRole('link', { name: 'Export settings', exact: true }).click();
  const download = await downloading;
  expect(download.suggestedFilename()).toBe('coinz-settings.json');
  const path = info.outputPath('synthetic-settings.json');
  await download.saveAs(path);
  expect(
    (await page.request.post('/api/logout', { headers: { 'X-CSRF-Token': csrf } })).ok(),
  ).toBeTruthy();
  await login('restore');
  const file = settings.getByLabel('Settings JSON file', { exact: true });
  await file.setInputFiles({
    name: 'future.json',
    mimeType: 'application/json',
    buffer: Buffer.from(JSON.stringify({ ...fixture, version: 2 })),
  });
  await settings.getByRole('button', { name: 'Preview settings', exact: true }).click();
  await expect(settings.getByRole('alert')).toContainText('Unsupported settings format or version');
  await file.setInputFiles(path);
  await settings.getByRole('button', { name: 'Preview settings', exact: true }).click();
  const preview = settings.getByRole('region', { name: 'Settings restore preview' });
  await expect(preview).toContainText('1 categories');
  await preview.getByRole('button', { name: 'Cancel restore', exact: true }).click();
  expect(
    (await (await page.request.get('/api/exports/settings.json')).json()).classifications,
  ).toEqual([]);
  await settings.getByRole('button', { name: 'Preview settings', exact: true }).click();
  await expect(preview).toBeVisible();
  await settings.screenshot({ path: info.outputPath('settings-restore-preview.png') });
  await preview.getByRole('button', { name: 'Restore settings', exact: true }).click();
  await expect(settings.getByRole('status').filter({ hasText: 'Settings restored' })).toBeVisible();
  await expect(file).toHaveValue('');
  await expect(file).toBeEnabled();
  const restored = await (await page.request.get('/api/exports/settings.json')).json();
  expect(restored.classifications[0]).toMatchObject({
    name: 'Portable dining',
    color: 'clay',
    archived: true,
  });
  expect(restored.budgets[0].savings_cents).toBe('12345');
  await file.setInputFiles(path);
  const retryPreview = settings.getByRole('button', { name: 'Preview settings', exact: true });
  await expect(retryPreview).toBeEnabled();
  await retryPreview.click();
  await expect(settings.getByRole('alert')).toContainText('empty account');
});
