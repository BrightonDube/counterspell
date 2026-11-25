// AUD-19: an anonymous solve must not POST the score — the endpoint answers 401
// and the browser logs it. The sign-in nudge shows without the request.
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ fetchDaily: vi.fn(), saveScore: vi.fn() }));

vi.mock('#lib/break/api.js', () => api);
vi.mock('$app/paths', () => ({ resolve: (/** @type {string} */ path) => path }));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/break/puzzle') }
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

import { session } from '#lib/auth/session.svelte.js';
import PuzzlePage from './+page.svelte';

const DAILY = {
  ok: true,
  date: '2026-11-25',
  puzzle: {
    rating: 800,
    themes: [],
    moves: 'e2e4 e7e5',
    fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
  }
};

async function solvePuzzle() {
  await screen.findByRole('heading', { name: 'Daily chess puzzle' });
  await fireEvent.input(screen.getByLabelText(/Move in UCI/), {
    target: { value: 'e7e5' }
  });
  await fireEvent.click(screen.getByRole('button', { name: 'Play' }));
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  session.user = null;
  session.capabilities = null;
  session.status = 'unknown';
});

describe('daily puzzle session handling', () => {
  it('never posts a score for an anonymous solve', async () => {
    session.status = 'signed_out';
    api.fetchDaily.mockResolvedValue(DAILY);
    render(PuzzlePage);

    await solvePuzzle();

    await screen.findByText('Sign in to save your score.');
    expect(api.saveScore).not.toHaveBeenCalled();
  });

  it('posts the score for a signed-in solve', async () => {
    session.status = 'signed_in';
    api.fetchDaily.mockResolvedValue(DAILY);
    api.saveScore.mockResolvedValue({ ok: true, status: 'recorded', score: 42 });
    render(PuzzlePage);

    await solvePuzzle();

    await waitFor(() => expect(api.saveScore).toHaveBeenCalledTimes(1));
    expect(api.saveScore).toHaveBeenCalledWith({
      game: 'chess_puzzle',
      date: '2026-11-25',
      solved: true,
      mistakes: 0
    });
    await screen.findByText('Score saved: 42.');
  });
});
