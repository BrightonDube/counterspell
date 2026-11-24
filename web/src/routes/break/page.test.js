// Tests for the break-room page's session-aware fetching (AQ06/AUD-19/AUD-20):
// a signed-out visit must not fire the scores request at all, and a signed-in
// visit reuses the shared session answer instead of probing with a 401.
import { cleanup, render, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
  fetchDaily: vi.fn(),
  fetchScores: vi.fn()
}));

vi.mock('#lib/break/api.js', () => api);
vi.mock('$app/paths', () => ({ resolve: (/** @type {string} */ path) => path }));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/break') }
}));
vi.mock('#lib/art/AnimatedArt.svelte', () => ({ default: () => {} }));

import { session } from '#lib/auth/session.svelte.js';
import BreakPage from './+page.svelte';

beforeEach(() => {
  api.fetchDaily.mockResolvedValue({ ok: true });
  api.fetchScores.mockResolvedValue({ ok: true, scores: [] });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  session.user = null;
  session.capabilities = null;
  session.status = 'unknown';
});

describe('break room fetching', () => {
  it('never calls the scores endpoint while signed out', async () => {
    session.status = 'signed_out';
    render(BreakPage);
    await waitFor(() => expect(api.fetchDaily).toHaveBeenCalled());
    // Give any unconditional second fetch a tick to fire before asserting it did not.
    await new Promise((r) => setTimeout(r, 20));
    expect(api.fetchScores).not.toHaveBeenCalled();
  });

  it('fetches scores only when the session says signed in', async () => {
    session.status = 'signed_in';
    render(BreakPage);
    await waitFor(() => expect(api.fetchScores).toHaveBeenCalledTimes(1));
  });
});
