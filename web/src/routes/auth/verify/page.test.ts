import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, expect, it, vi } from 'vitest';
import { goto, replaceState } from '$app/navigation';
import Page from './+page.svelte';

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
  replaceState: vi.fn()
}));

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
  window.history.replaceState(null, '', '/');
});

it('hides the email link token before confirmation and opens two-factor sign-in when required', async () => {
  const fetch = vi
    .fn()
    .mockResolvedValue(Response.json({ return_to: '/login?step=2fa' }));
  vi.stubGlobal('fetch', fetch);
  window.history.replaceState(null, '', '/auth/verify#token=fixture-token');
  render(Page);

  const continueButton = await screen.findByRole('button', {
    name: 'Continue'
  });
  expect(replaceState).toHaveBeenCalledWith('/auth/verify', {});
  expect(fetch).not.toHaveBeenCalled();

  await fireEvent.click(continueButton);
  await waitFor(() =>
    expect(goto).toHaveBeenCalledWith('/login?step=2fa', { replaceState: true })
  );
  expect(fetch).toHaveBeenCalledWith(
    '/api/auth/email/verify',
    expect.objectContaining({
      method: 'POST',
      body: '{"token":"fixture-token"}'
    })
  );
});
