import { expect, test } from '@playwright/test';

test('categories and labels stay distinct through editing, filtering and archiving', async ({
  page,
}, info) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await page.goto('/');
  await page.getByLabel('Username', { exact: true }).fill(`${info.project.name}-user`);
  await page.getByLabel('Password', { exact: true }).fill('synthetic-browser-password');
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'The bigger picture.' })).toBeVisible();
  await page.getByRole('link', { name: 'Organize', exact: true }).click();
  await page.getByRole('button', { name: 'Start empty', exact: true }).click();
  await page.getByLabel('New category', { exact: true }).fill('Dining fixture');
  await page.getByRole('button', { name: 'Create category', exact: true }).click();
  await expect(
    page.getByRole('button', { name: 'Edit Dining fixture', exact: true }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  await page.getByLabel('New label', { exact: true }).fill('Italy fixture');
  await page.getByLabel('Colour', { exact: true }).selectOption('blue');
  await page.getByRole('button', { name: 'Create label', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Edit Italy fixture', exact: true })).toBeVisible();
  await page.getByRole('link', { name: 'Entries', exact: true }).click();
  await page.getByRole('button', { name: '+ Add transaction', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'Add a transaction' });
  await dialog.getByLabel('Description', { exact: true }).fill('Synthetic holiday dinner');
  await dialog.getByLabel('Amount (€)', { exact: true }).fill('40');
  await dialog.getByLabel('Date', { exact: true }).fill('2026-11-07');
  await dialog.getByRole('button', { name: /Change category for this entry/ }).click();
  await dialog.getByRole('button', { name: 'Dining fixture', exact: true }).click();
  await dialog.getByRole('button', { name: 'Add labels to this entry', exact: true }).click();
  await dialog.getByRole('button', { name: '+ Italy fixture', exact: true }).click();
  await dialog.getByRole('button', { name: 'Done', exact: true }).click();
  await dialog.getByRole('button', { name: 'Record transaction', exact: true }).click();
  await expect(dialog).not.toBeVisible();
  await page.getByRole('radio', { name: 'Custom', exact: true }).check();
  const row = page
    .getByRole('list', { name: 'Recorded entries' })
    .getByRole('listitem')
    .filter({ hasText: 'Synthetic holiday dinner' });
  await expect(row.getByRole('button', { name: /Change category.*Dining fixture/ })).toBeVisible();
  const contentBox = await row.locator('.entry-content').boundingBox();
  const classificationBox = await row.locator('.classification-line').boundingBox();
  expect(contentBox).not.toBeNull();
  expect(classificationBox).not.toBeNull();
  if (info.project.name === 'desktop') {
    expect(classificationBox!.x).toBeGreaterThanOrEqual(contentBox!.x + contentBox!.width);
  } else {
    expect(classificationBox!.y).toBeGreaterThanOrEqual(contentBox!.y + contentBox!.height);
  }

  await row
    .getByRole('button', { name: 'Add labels to Synthetic holiday dinner', exact: true })
    .click();
  await row.getByRole('button', { name: 'All labels…', exact: true }).click();
  await row.getByLabel('Find or create a label').fill('Shared fixture');
  await row.getByRole('button', { name: 'Create “Shared fixture”', exact: true }).click();
  await expect(
    row.getByRole('button', { name: 'Remove label Shared fixture', exact: true }),
  ).toBeAttached();
  await expect(row.getByRole('region', { name: 'Choose labels' })).toBeVisible();
  await row.getByRole('button', { name: 'Done', exact: true }).click();
  await row.locator('.label-chip').filter({ hasText: 'Shared fixture' }).hover();
  await row.getByRole('button', { name: 'Remove label Shared fixture', exact: true }).click();
  await expect(
    row.getByRole('button', { name: 'Remove label Shared fixture', exact: true }),
  ).toHaveCount(0);
  const filters = page.getByRole('region', { name: 'Classification filters' });
  await filters.getByRole('button', { name: /^Labels/ }).click();
  await filters.getByLabel('Italy fixture', { exact: true }).check();
  await expect(page.getByTestId('spending')).toHaveText('€40.00');
  await page.screenshot({
    path: `test-results/classification-${info.project.name}.png`,
    fullPage: true,
  });
  await page.getByRole('link', { name: 'Organize', exact: true }).click();
  await page.getByRole('button', { name: 'Labels', exact: true }).click();
  await page.getByRole('button', { name: 'Archive Italy fixture', exact: true }).click();
  await page.getByLabel('Include archived', { exact: true }).check();
  await expect(
    page.getByRole('button', { name: 'Restore Italy fixture', exact: true }),
  ).toBeVisible();
  await page.getByRole('link', { name: 'Entries', exact: true }).click();
  await page.getByRole('radio', { name: 'Custom', exact: true }).check();
  await expect(
    page.getByRole('button', { name: 'Remove label Italy fixture', exact: true }),
  ).toBeAttached();
  expect(errors).toEqual([]);
});
