// Browser end-to-end tests. They run against `counterspell-testserver`: the production Axum
// router, a real Postgres database, the built static site and a scripted judge. Judging is
// scripted (every run passes), so these tests check the dashboard, routing and states, never
// judge behaviour. `DATABASE_URL` comes from the caller's environment and is never written here.
//
//   npm run build && DATABASE_URL=... npm run e2e
import { defineConfig, devices } from '@playwright/test';

const PORT = 5198;

export default defineConfig({
  testDir: './e2e',
  // One worker: the test server's per-user run limit is counted over a 600 s window.
  workers: 1,
  fullyParallel: false,
  retries: 0,
  reporter: [['list']],
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    ...devices['Desktop Chrome'],
    // Set E2E_EXECUTABLE to use a Chromium that was installed for another Playwright version
    // (for example ~/.cache/ms-playwright/chromium_headless_shell-1255/.../chrome-headless-shell).
    ...(process.env.E2E_EXECUTABLE
      ? { launchOptions: { executablePath: process.env.E2E_EXECUTABLE } }
      : {})
  },
  webServer: {
    command: 'cargo run -q -p counterspell-testserver',
    cwd: '..',
    url: `http://127.0.0.1:${PORT}/api/health`,
    reuseExistingServer: false,
    timeout: 300_000,
    env: {
      ...process.env,
      CS_TEST_PORT: String(PORT),
      // The browser always connects from loopback, so lift the per-IP cap. The per-user cap stays.
      RUN_RATE_IP: '10000',
      STATIC_DIR: 'web/build'
    }
  }
});
