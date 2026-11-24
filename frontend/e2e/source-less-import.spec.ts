import { expect, test } from '@playwright/test';
test('CSV import works without creating or choosing a source', async ({ page }, info) => {
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${info.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
  const token = (await (await page.request.get('/api/session')).json()).csrf_token;
  const profile = await page.request.post('/api/imports/profiles', {
    headers: { 'X-CSRF-Token': token },
    data: {
      name: 'No source fixture',
      layout: {
        date: 'date',
        amount: 'amount',
        description: 'description',
        external_id: 'external_id',
        delimiter: ',',
        date_format: 'YYYY-MM-DD',
        decimal: '.',
        grouping: '',
      },
    },
  });
  expect(profile.ok()).toBeTruthy();
  const profileId = (await profile.json()).id;
  await page.getByRole('button', { name: 'Import CSV', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Import CSV', exact: true });
  await expect(dialog.getByLabel('Import source', { exact: true })).not.toBeVisible();
  await dialog.getByLabel('Import profile', { exact: true }).selectOption(profileId);
  await dialog.getByLabel('CSV file').setInputFiles({
    name: 'source-less-synthetic.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from(
      'date,description,amount,external_id\n2026-11-09,Standalone source-less fixture,-9.87,source-less-1\n',
    ),
  });
  await dialog.getByRole('button', { name: 'Review entries', exact: true }).click();
  await expect(
    dialog.getByRole('heading', { name: 'Review 1 entries', exact: true }),
  ).toBeVisible();
  await expect(dialog).toContainText('No source');
  await page.screenshot({ path: info.outputPath('import-without-source.png'), fullPage: true });
  const commit = dialog.getByRole('button', { name: /^Import [0-9]+ entries/ });
  await commit.click();
  await expect(dialog).not.toBeVisible();
  const entries = await (
    await page.request.get('/api/entries?q=Standalone%20source-less%20fixture')
  ).json();
  expect(entries.entries).toHaveLength(1);
  expect(entries.entries[0].source_id).toBeNull();
});
