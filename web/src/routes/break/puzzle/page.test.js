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
  props: undefined,
  /** @type {[any, string][]} */
  added: /** @type {[any, string][]} */ ([])
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
    /** @param {any} type @param {string} square */
    addMarker(type, square) {
      boardMock.added.push([type, square]);
    }
    removeMarkers() {}
  },
  INPUT_EVENT_TYPE: { validateMoveInput: 'validateMoveInput' }
}));
vi.mock('cm-chessboard/assets/chessboard.css', () => ({}));
vi.mock('cm-chessboard/src/extensions/markers/Markers.js', () => ({
  Markers: function Markers() {},
  MARKER_TYPE: {
    frame: { class: 'marker-frame', slice: 'markerFrame' },
    framePrimary: { class: 'marker-frame-primary', slice: 'markerFrame' }
  }
}));
const themeMock = vi.hoisted(() => ({
  /** @type {Promise<void> | undefined} */
  gate: undefined,
  called: false
}));

vi.mock('#lib/break/chess/theme.js', async (importOriginal) => {
  const actual = /** @type {typeof import('#lib/break/chess/theme.js')} */ (
    await importOriginal()
  );
  return {
    ...actual,
    mountThemedBoard: async (
      /** @type {HTMLElement | undefined} */ el,
      /** @type {string | (() => string)} */ position,
      /** @type {'w' | 'b'} */ orientation
    ) => {
      themeMock.called = true;
      await themeMock.gate;
      return actual.mountThemedBoard(el, position, orientation);
    }
  };
});

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
  boardMock.added.length = 0;
  themeMock.gate = undefined;
  themeMock.called = false;
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

  it('frames the opponent opening and the applied solution moves', async () => {
    session.status = 'signed_in';
    api.fetchDaily.mockResolvedValue(DAILY);
    render(PuzzlePage);
    await screen.findByText('Your move.');
    const frame = { class: 'marker-frame-primary', slice: 'markerFrame' };
    // The automatic opening move (e2e4) is framed once the board mounts.
    await waitFor(() => {
      expect(boardMock.added).toContainEqual([frame, 'e2']);
      expect(boardMock.added).toContainEqual([frame, 'e4']);
    });
    boardMock.added.length = 0;
    await solvePuzzle();
    await waitFor(() => {
      expect(boardMock.added).toContainEqual([frame, 'e7']);
      expect(boardMock.added).toContainEqual([frame, 'e5']);
    });
  });

  it('syncs a solution applied while the board imports are pending', async () => {
    session.status = 'signed_in';
    api.fetchDaily.mockResolvedValue(DAILY);
    /** @type {() => void} */
    let release = () => {};
    themeMock.gate = new Promise((resolve) => {
      release = resolve;
    });
    render(PuzzlePage);
    // The puzzle is loaded and the text input live while the lazy board
    // import is still in flight — a solve here must not be lost.
    await waitFor(() => expect(themeMock.called).toBe(true));
    await solvePuzzle();
    release();
    await waitFor(() => expect(boardMock.props).toBeTruthy());
    // e7e5 solved the puzzle during the wait: the board must mount showing
    // White to move, not the post-opening FEN captured before the imports.
    expect(boardMock.props?.position).toContain(' w ');
    const frame = { class: 'marker-frame-primary', slice: 'markerFrame' };
    await waitFor(() => {
      expect(boardMock.added).toContainEqual([frame, 'e7']);
      expect(boardMock.added).toContainEqual([frame, 'e5']);
    });
  });
});
