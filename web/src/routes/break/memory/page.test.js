// AUD-19: an anonymous player losing a Lantern recall round must not trigger a
// scores POST — the endpoint answers 401 and the browser logs it as noise.
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ saveScore: vi.fn() }));

vi.mock('#lib/break/api.js', () => api);
vi.mock('$app/paths', () => ({
  resolve: (/** @type {string} */ path) => path
}));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/break/memory') }
}));
// Deterministic sequence: every round grows with tile 0, so pressing tile 2 is
// always the losing move.
vi.mock('#lib/break/memory.js', () => ({
  newRound: (/** @type {number[]} */ seq) => seq.concat([0]),
  checkPress: (
    /** @type {number[]} */ seq,
    /** @type {number[]} */ presses,
    /** @type {number} */ i
  ) =>
    presses[i] === seq[i]
      ? presses.length === seq.length
        ? 'round_done'
        : 'correct'
      : 'wrong',
  level: (/** @type {number[]} */ seq) => seq.length
}));

import { session } from '#lib/auth/session.svelte.js';
import MemoryPage from './+page.svelte';

/** @param {boolean} reduced */
function stubMatchMedia(reduced) {
  vi.stubGlobal('matchMedia', () => ({ matches: reduced }));
}

async function playOneWrongMove() {
  await fireEvent.click(screen.getByRole('button', { name: 'Start' }));
  await screen.findByText('Your turn. Repeat the sequence.');
  await fireEvent.click(screen.getByRole('button', { name: 'Tile 2' }));
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.unstubAllGlobals();
  session.user = null;
  session.capabilities = null;
  session.status = 'unknown';
});

describe('lantern recall session handling', () => {
  it('never posts a score for an anonymous player', async () => {
    stubMatchMedia(true);
    session.status = 'signed_out';
    api.saveScore.mockResolvedValue({ ok: false, status: 401 });
    render(MemoryPage);

    await playOneWrongMove();

    await waitFor(() =>
      expect(screen.getByRole('link', { name: 'Sign in' })).toBeTruthy()
    );
    expect(api.saveScore).not.toHaveBeenCalled();
  });

  it('saves the level for a signed-in player', async () => {
    stubMatchMedia(true);
    session.status = 'signed_in';
    api.saveScore.mockResolvedValue({ ok: true });
    render(MemoryPage);

    await playOneWrongMove();

    await waitFor(() => expect(api.saveScore).toHaveBeenCalledTimes(1));
    expect(api.saveScore).toHaveBeenCalledWith({ game: 'brain', level: 3 });
  });
});
