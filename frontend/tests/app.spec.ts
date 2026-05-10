import { expect, test } from '@playwright/test';

test('renders primary collection tabs', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Consoles' })).toBeVisible();
  await expect(page.getByRole('button', { name: /Games/ }).first()).toBeVisible();
});

test('filters games without losing the app shell', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: /Games/ }).first().click();
  await page.getByPlaceholder('Search collection').fill('Mario');
  await expect(page.getByRole('heading', { name: 'Games' })).toBeVisible();
});

