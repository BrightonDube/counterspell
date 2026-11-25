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
  level: (/** @type {number[]} */ seq) => seq.length,
  TILE_ART: Array.from(
    { length: 9 },
    (/** @type {any} */ _, /** @type {number} */ i) =>
      `/art/objects/test-${i}.webp`
  )
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

describe('themed tile reveals (LR02–LR05)', () => {
  it('renders every tile with a decorative lazy-loaded image', () => {
    stubMatchMedia(true);
    const { container } = render(MemoryPage);
    const arts = container.querySelectorAll('.tile-art');
    expect(arts.length).toBe(9);
    for (const img of arts) {
      expect(img.getAttribute('loading')).toBe('lazy');
      expect(img.getAttribute('fetchpriority')).toBe('low');
      expect(img.getAttribute('aria-hidden')).toBe('true');
      expect(img.getAttribute('alt')).toBe('');
      expect(img.getAttribute('src')).toMatch(/^\/art\/objects\/.+\.webp$/);
    }
  });

  it('reveals the image beneath the lit tile during playback', async () => {
    vi.useFakeTimers();
    try {
      stubMatchMedia(false);
      const { container } = render(MemoryPage);
      await fireEvent.click(screen.getByRole('button', { name: 'Start' }));
      // The mock grows every round with tile 0, so tile 1 lights first.
      const lit = container.querySelector('.tile-reveal');
      expect(lit).toBeTruthy();
      expect(lit?.getAttribute('aria-label')).toBe('Tile 1');
      await vi.advanceTimersByTimeAsync(600);
      expect(container.querySelector('.tile-reveal')).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  it('reveals the pressed tile image before the verdict flash', async () => {
    vi.useFakeTimers();
    try {
      stubMatchMedia(true);
      const { container } = render(MemoryPage);
      await fireEvent.click(screen.getByRole('button', { name: 'Start' }));
      await vi.advanceTimersByTimeAsync(400); // reduced playback → 'input'
      await fireEvent.click(screen.getByRole('button', { name: 'Tile 1' }));
      // The image reveals immediately; the verdict colour lands after it.
      expect(container.querySelector('.tile-reveal')).toBeTruthy();
      expect(container.querySelector('.tile-correct')).toBeNull();
      expect(screen.getByText('Correct.')).toBeTruthy();
      await vi.advanceTimersByTimeAsync(200);
      expect(container.querySelector('.tile-correct')).toBeTruthy();
    } finally {
      vi.useRealTimers();
    }
  });

  it('reveals the pressed tile image before wrong feedback', async () => {
    vi.useFakeTimers();
    try {
      stubMatchMedia(true);
      const { container } = render(MemoryPage);
      await fireEvent.click(screen.getByRole('button', { name: 'Start' }));
      await vi.advanceTimersByTimeAsync(400);
      await fireEvent.click(screen.getByRole('button', { name: 'Tile 2' }));
      expect(container.querySelector('.tile-reveal')).toBeTruthy();
      expect(container.querySelector('.tile-wrong')).toBeNull();
      expect(screen.getByText('Wrong, game over at level 3.')).toBeTruthy();
      await vi.advanceTimersByTimeAsync(200);
      expect(container.querySelector('.tile-wrong')).toBeTruthy();
    } finally {
      vi.useRealTimers();
    }
  });

  it('keeps roving focus and keypress play intact', async () => {
    vi.useFakeTimers();
    try {
      stubMatchMedia(true);
      const { container } = render(MemoryPage);
      await fireEvent.click(screen.getByRole('button', { name: 'Start' }));
      await vi.advanceTimersByTimeAsync(400);
      const tile2 = screen.getByRole('button', { name: 'Tile 2' });
      const tile3 = screen.getByRole('button', { name: 'Tile 3' });
      await fireEvent.focus(tile2);
      await fireEvent.keyDown(tile2, { key: 'ArrowRight' });
      expect(tile3.getAttribute('tabindex')).toBe('0');
      expect(tile2.getAttribute('tabindex')).toBe('-1');
      // Enter presses the focused tile — the reveal still precedes feedback.
      await fireEvent.keyDown(tile3, { key: 'Enter' });
      expect(container.querySelector('.tile-reveal')).toBeTruthy();
      expect(container.querySelector('.tile-wrong')).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });
});
