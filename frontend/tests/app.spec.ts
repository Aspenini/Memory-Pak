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

async function shellFrame(page: Page): Promise<Record<string, { x: number; y: number; width: number; height: number }>> {
  return page.evaluate(() => {
    const frameFor = (selector: string) => {
      const rect = document.querySelector(selector)?.getBoundingClientRect();
      if (!rect) throw new Error(`${selector} not found`);
      return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
    };
    return {
      topbar: frameFor('.topbar'),
      toolbar: frameFor('.toolbar'),
      viewport: frameFor('.list-viewport')
    };
  });
}

async function expectNoHorizontalOverflow(locator: Locator): Promise<void> {
  const overflow = await locator.evaluate((element) => element.scrollWidth - element.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
}

function expectStableFrame(
  expected: Record<string, { x: number; y: number; width: number; height: number }>,
  actual: Record<string, { x: number; y: number; width: number; height: number }>
): void {
  for (const region of Object.keys(expected)) {
    for (const key of ['x', 'y', 'width', 'height'] as const) {
      expect(Math.abs(actual[region][key] - expected[region][key])).toBeLessThanOrEqual(1);
    }
  }
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

test('keeps the desktop shell stable while filtering and sorting', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop');
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Consoles' })).toBeVisible();
  await page.waitForTimeout(250);
  await expect(page.locator('.sidebar-actions')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Updates' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Backup' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Restore' })).toBeVisible();
  await expectNoHorizontalOverflow(page.locator('.topbar'));
  for (const action of await page.locator('.sidebar-actions button').all()) {
    await expectNoHorizontalOverflow(action);
  }

  const initial = await shellFrame(page);
  await page.getByRole('tab', { name: 'Owned', exact: true }).click();
  await page.waitForTimeout(300);
  expectStableFrame(initial, await shellFrame(page));

  await page.getByRole('button', { name: /Sort/ }).click();
  await page.getByRole('option', { name: 'Status' }).click();
  await page.waitForTimeout(300);
  expectStableFrame(initial, await shellFrame(page));

  await clickInViewportGames(page);
  await page.waitForTimeout(300);
  expectStableFrame(initial, await shellFrame(page));
});
