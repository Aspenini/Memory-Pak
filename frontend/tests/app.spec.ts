import { expect, test, type Locator, type Page } from '@playwright/test';

async function clickInViewportGames(page: Page): Promise<void> {
  const buttons = page.getByRole('button', { name: /Games/ });
  await buttons.first().waitFor({ state: 'attached' });
  const viewport = page.viewportSize();
  if (!viewport) throw new Error('viewport size is required');

  const candidates = await buttons.all();
  for (const candidate of candidates) {
    if (await isInViewport(candidate, viewport.width, viewport.height)) {
      await candidate.click();
      return;
    }
  }
  throw new Error('no in-viewport Games button found');
}

async function isInViewport(locator: Locator, w: number, h: number): Promise<boolean> {
  const box = await locator.boundingBox();
  if (!box) return false;
  return box.x >= 0 && box.y >= 0 && box.x + box.width <= w && box.y + box.height <= h;
}

test('renders primary collection tabs', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Consoles' })).toBeVisible();
  await expect(page.getByRole('button', { name: /Games/ }).first()).toBeAttached();
});

test('filters games without losing the app shell', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Consoles' })).toBeVisible();
  await clickInViewportGames(page);
  await page.getByPlaceholder('Search collection').fill('Mario');
  await expect(page.getByRole('heading', { name: 'Games' })).toBeVisible();
});

