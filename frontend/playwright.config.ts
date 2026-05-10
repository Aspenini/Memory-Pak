import { defineConfig, devices } from '@playwright/test';

const baseURL = 'http://127.0.0.1:5173/Memory-Pak/app/';

export default defineConfig({
  testDir: './tests',
  webServer: {
    command: 'bun run dev:e2e',
    url: baseURL,
    reuseExistingServer: !process.env.CI
  },
  use: {
    baseURL
  },
  projects: [
    {
      name: 'desktop',
      use: { ...devices['Desktop Chrome'] }
    },
    {
      name: 'mobile',
      use: { ...devices['Pixel 7'] }
    }
  ]
});
