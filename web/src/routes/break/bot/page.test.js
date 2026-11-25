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
vi.mock('#lib/break/chess/pieces-steel-night.svg?url', () => ({
  default: '/assets/pieces-steel-night.svg'
}));
vi.mock('cm-chessboard/assets/extensions/markers/markers.svg?url', () => ({
  default: '/assets/markers.svg'
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
  boardMock.props = undefined;
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
});
