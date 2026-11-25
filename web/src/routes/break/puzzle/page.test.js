// AUD-19: an anonymous solve must not POST the score — the endpoint answers 401
// and the browser logs it. The sign-in nudge shows without the request.
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ fetchDaily: vi.fn(), saveScore: vi.fn() }));

vi.mock('#lib/break/api.js', () => api);
vi.mock('$app/paths', () => ({
  resolve: (/** @type {string} */ path) => path
}));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/break/puzzle') }
}));
const boardMock = vi.hoisted(() => ({
  /** @type {Record<string, any> | undefined} */
  props: undefined
}));

vi.mock('cm-chessboard', () => ({
  Chessboard: class {
    /**
     * @param {HTMLElement} _el
     * @param {Record<string, any>} props
     */
    constructor(_el, props) {
      boardMock.props = props;
    }
    setPosition() {}
    enableMoveInput() {}
  },
  INPUT_EVENT_TYPE: { validateMoveInput: 'validateMoveInput' }
}));
vi.mock('cm-chessboard/assets/chessboard.css', () => ({}));
vi.mock('cm-chessboard/assets/pieces/standard.svg?url', () => ({
  default: 'pieces.svg'
}));
vi.mock('cm-chessboard/src/extensions/markers/Markers.js', () => ({
  Markers: function Markers() {},
  MARKER_TYPE: { frame: { class: 'marker-frame', slice: 'markerFrame' } }
}));
vi.mock('#lib/chess/pieces-steel-night.svg?url', () => ({
  default: '/assets/pieces-steel-night.svg'
}));
vi.mock('cm-chessboard/assets/extensions/markers/markers.svg?url', () => ({
  default: '/assets/markers.svg'
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
  boardMock.props = undefined;
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
    api.saveScore.mockResolvedValue({
      ok: true,
      status: 'recorded',
      score: 42
    });
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

describe('steel-night board presentation (CH02–CH06)', () => {
  it('mounts the board with the shared steel-night style and markers', async () => {
    session.status = 'signed_in';
    api.fetchDaily.mockResolvedValue(DAILY);
    render(PuzzlePage);
    await screen.findByRole('img', { name: 'Chessboard' });
    await waitFor(() => expect(boardMock.props).toBeTruthy());
    expect(boardMock.props?.style?.cssClass).toBe('steel-night');
    expect(boardMock.props?.style?.pieces?.file).toMatch(/steel-night.*\.svg$/);
    expect(boardMock.props?.extensions?.length).toBeGreaterThan(0);
  });

  it('shows the skull opponent while the puzzle is playable', async () => {
    session.status = 'signed_in';
    api.fetchDaily.mockResolvedValue(DAILY);
    const { container } = render(PuzzlePage);
    expect(container.querySelector('.skull-sentinel')).toBeNull();
    await screen.findByText('Your move.');
    const skull = container.querySelector('.skull-sentinel');
    expect(skull).toBeTruthy();
    expect(skull?.getAttribute('aria-hidden')).toBe('true');
  });
});
