// Tests for the chess-bot page's session-aware copy (AQ05/AUD-18): the sign-in
// nudge is for anonymous visitors only — a signed-in player must never be told
// their results are not saved.
import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ saveScore: vi.fn() }));

vi.mock('#lib/break/api.js', () => api);
vi.mock('$app/paths', () => ({
  resolve: (/** @type {string} */ path) => path
}));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/break/bot') }
}));
vi.mock('cm-chessboard', () => ({
  Chessboard: class {
    setPosition() {}
    enableMoveInput() {}
  },
  INPUT_EVENT_TYPE: { validateMoveInput: 'validateMoveInput' }
}));
vi.mock('cm-chessboard/assets/chessboard.css', () => ({}));
vi.mock('cm-chessboard/assets/pieces/standard.svg?url', () => ({
  default: 'pieces.svg'
}));
vi.mock('#lib/break/bot.worker.js?worker', () => ({
  default: class {
    postMessage() {}
    terminate() {}
  }
}));

import { session } from '#lib/auth/session.svelte.js';
import BotPage from './+page.svelte';

beforeEach(() => {
  api.saveScore.mockResolvedValue({ ok: true });
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  session.user = null;
  session.capabilities = null;
  session.status = 'unknown';
});

describe('chess bot session copy', () => {
  it('does not tell a signed-in player their results are unsaved', async () => {
    session.status = 'signed_in';
    render(BotPage);
    await screen.findByRole('heading', { name: 'Chess bot' });
    await waitFor(() => {
      expect(
        screen.queryByText(/Signed-out results are not saved/i)
      ).toBeNull();
      expect(screen.queryByRole('link', { name: 'Sign in' })).toBeNull();
    });
  });

  it('invites an anonymous player to sign in', async () => {
    session.status = 'signed_out';
    render(BotPage);
    await screen.findByRole('heading', { name: 'Chess bot' });
    await screen.findByText(
      (_content, el) =>
        el?.tagName === 'P' &&
        /Sign in\s+to save your results/i.test(el.textContent ?? '')
    );
    const link = screen.getByRole('link', { name: 'Sign in' });
    expect(link.getAttribute('href')).toBe('/login?return_to=/break/bot');
  });

  // Breaks if the board loses its one accessible name — moves are already
  // announced by the status line, so the board itself just needs a name (AUD-14).
  it('exposes the chessboard as a named image', async () => {
    session.status = 'signed_in';
    render(BotPage);
    await screen.findByRole('heading', { name: 'Chess bot' });
    expect(screen.getByRole('img', { name: 'Chessboard' })).toBeTruthy();
  });
});
