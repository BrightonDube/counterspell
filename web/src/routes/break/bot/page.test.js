// Tests for the chess-bot page's session-aware copy (AQ05/AUD-18): the sign-in
// nudge is for anonymous visitors only — a signed-in player must never be told
// their results are not saved.
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({ saveScore: vi.fn() }));

vi.mock('#lib/break/api.js', () => api);
vi.mock('$app/paths', () => ({
  resolve: (/** @type {string} */ path) => path
}));
vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/break/bot') }
}));
const boardMock = vi.hoisted(() => ({
  /** @type {Record<string, any> | undefined} */
  props: undefined,
  /** @type {((event: any) => boolean) | undefined} */
  inputHandler: undefined,
  /** @type {[any, string][]} */
  added: /** @type {[any, string][]} */ ([]),
  /** @type {any[]} */
  removed: /** @type {any[]} */ ([])
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
    /** @param {(event: any) => boolean} handler */
    enableMoveInput(handler) {
      boardMock.inputHandler = handler;
    }
    /** @param {any} type @param {string} square */
    addMarker(type, square) {
      boardMock.added.push([type, square]);
    }
    /** @param {any} type */
    removeMarkers(type) {
      boardMock.removed.push(type);
    }
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
const workerMock = vi.hoisted(() => ({
  /** @type {{ onmessage?: (event: any) => void } | undefined} */
  instance: undefined
}));

vi.mock('#lib/break/bot.worker.js?worker', () => ({
  default: class {
    constructor() {
      workerMock.instance = /** @type {any} */ (this);
    }
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
  boardMock.props = undefined;
  boardMock.inputHandler = undefined;
  boardMock.added.length = 0;
  boardMock.removed.length = 0;
  workerMock.instance = undefined;
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

describe('steel-night board presentation (CH02–CH06)', () => {
  it('mounts the board with the shared steel-night style and markers', async () => {
    session.status = 'signed_in';
    render(BotPage);
    await screen.findByRole('img', { name: 'Chessboard' });
    await waitFor(() => expect(boardMock.props).toBeTruthy());
    expect(boardMock.props?.style?.cssClass).toBe('steel-night');
    expect(boardMock.props?.style?.pieces?.file).toMatch(/steel-night.*\.svg$/);
    expect(boardMock.props?.extensions?.length).toBeGreaterThan(0);
  });

  it('shows the skull opponent only while play is active', async () => {
    session.status = 'signed_in';
    const { container } = render(BotPage);
    await screen.findByRole('img', { name: 'Chessboard' });
    expect(container.querySelector('.skull-sentinel')).toBeNull();
    await fireEvent.click(screen.getByRole('button', { name: 'Play White' }));
    const skull = container.querySelector('.skull-sentinel');
    expect(skull).toBeTruthy();
    expect(skull?.getAttribute('aria-hidden')).toBe('true');
    expect(skull?.getAttribute('loading')).toBe('lazy');
  });

  it('frames the accepted user move and the bot reply', async () => {
    session.status = 'signed_in';
    render(BotPage);
    await screen.findByRole('img', { name: 'Chessboard' });
    await waitFor(() => expect(boardMock.inputHandler).toBeTypeOf('function'));
    await fireEvent.click(screen.getByRole('button', { name: 'Play White' }));
    const frame = { class: 'marker-frame-primary', slice: 'markerFrame' };
    const accepted = boardMock.inputHandler?.({
      type: 'validateMoveInput',
      squareFrom: 'e2',
      squareTo: 'e4',
      chessboard: {}
    });
    expect(accepted).toBe(true);
    expect(boardMock.added).toContainEqual([frame, 'e2']);
    expect(boardMock.added).toContainEqual([frame, 'e4']);
    // The worker's reply lands through onmessage and must be framed too.
    await waitFor(() =>
      expect(workerMock.instance?.onmessage).toBeTypeOf('function')
    );
    workerMock.instance?.onmessage?.({ data: { ok: true, uci: 'e7e5' } });
    await waitFor(() => {
      expect(boardMock.added).toContainEqual([frame, 'e7']);
      expect(boardMock.added).toContainEqual([frame, 'e5']);
    });
  });

  it('clears the last-move frame when a new game starts', async () => {
    session.status = 'signed_in';
    render(BotPage);
    await screen.findByRole('img', { name: 'Chessboard' });
    await waitFor(() => expect(boardMock.inputHandler).toBeTypeOf('function'));
    await fireEvent.click(screen.getByRole('button', { name: 'Play White' }));
    expect(boardMock.removed).toContainEqual({
      class: 'marker-frame-primary',
      slice: 'markerFrame'
    });
  });
});
