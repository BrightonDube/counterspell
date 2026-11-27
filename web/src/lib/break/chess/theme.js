// Shared cm-chessboard presentation for the bot and puzzle routes (CH03–CH05):
// one Steel Night theme class, the original piece sprite, framed coordinates
// and the last-move marker — so both boards look and behave identically.
//
// Sprites are literal same-origin paths under web/static/break/: cm-chessboard
// fetches them with XMLHttpRequest, so a `?url` import that Vite inlines to a
// `data:` URL would violate the deployed `connect-src 'self'` policy (the
// marker sprite is below Vite's 4 kB inlining threshold).
const PIECES_URL = '/break/pieces-obsidian-ivory.svg';
const MARKERS_URL = '/break/markers.svg';

/** Style contract shared by every Counterspell board. */
export const CHESSBOARD_STYLE = Object.freeze({
  cssClass: 'steel-night',
  showCoordinates: true,
  borderType: 'frame',
  aspectRatio: 1,
  pieces: { tileSize: 40, file: PIECES_URL },
  animationDuration: 300
});

/**
 * Last-move highlight type. Deliberately a literal `{class, slice}` record —
 * cm-chessboard marker types are data, and `framePrimary` shares the frame
 * slice while staying a distinct type the extension's `autoMarkers`
 * (`marker-frame`) never clears during move validation.
 */
export const LAST_MOVE_MARKER = Object.freeze({
  class: 'marker-frame-primary',
  slice: 'markerFrame'
});

/**
 * Frames both squares of the last applied move — the user's accepted move, a
 * bot reply, or a puzzle auto-response. Clears the previous frame first.
 * @param {Pick<import('cm-chessboard').Chessboard, 'addMarker' | 'removeMarkers'> | undefined} board
 * @param {string} from
 * @param {string} to
 */
export function showLastMove(board, from, to) {
  board?.removeMarkers?.(LAST_MOVE_MARKER);
  board?.addMarker?.(LAST_MOVE_MARKER, from);
  board?.addMarker?.(LAST_MOVE_MARKER, to);
}

/**
 * Clears the last-move frame (new game / new puzzle).
 * @param {Pick<import('cm-chessboard').Chessboard, 'removeMarkers'> | undefined} board
 */
export function clearLastMove(board) {
  board?.removeMarkers?.(LAST_MOVE_MARKER);
}

/**
 * In-flight orientation turns per board. cm-chessboard 8.x drops a
 * `setOrientation` call outright while a turn is queued (`boardTurning`
 * guard + console warning), so a colour picked during a pending turn would
 * silently never apply. Requests are serialized here instead: each call
 * chains behind the in-flight one and re-issues with the newest side.
 * @type {WeakMap<object, Promise<void>>}
 */
const pendingTurns = new WeakMap();

/**
 * Flips the board so `color` plays from the bottom. cm-chessboard exposes
 * `setOrientation` at runtime but its shipped types omit it, so the call
 * goes through a narrow structural type here.
 * @param {import('cm-chessboard').Chessboard | undefined} board
 * @param {'w' | 'b'} color
 */
export function setBoardOrientation(board, color) {
  /** @type {any} */
  const mutable = board;
  if (!board || !mutable.setOrientation) return;
  const pending = pendingTurns.get(board);
  if (pending) {
    pendingTurns.set(
      board,
      pending.finally(() => setBoardOrientation(board, color))
    );
    return;
  }
  if (mutable.getOrientation?.() === color) return;
  pendingTurns.set(
    board,
    Promise.resolve(mutable.setOrientation(color))
      .catch(() => {})
      .finally(() => pendingTurns.delete(board))
  );
}

/**
 * Mounts a themed board in `container`. cm-chessboard ships `main: null`, so
 * both imports stay lazy — the routes remain server-renderable. The Markers
 * extension adds the selection frames on top.
 * @param {HTMLElement | undefined} container
 * @param {string | (() => string)} position FEN to show — a getter is
 *   evaluated only after the lazy imports resolve, so a game that advanced
 *   during the wait still mounts its current position.
 * @param {'w' | 'b'} [orientation]
 */
export async function mountThemedBoard(container, position, orientation = 'w') {
  const [{ Chessboard, INPUT_EVENT_TYPE }, markers] = await Promise.all([
    import('cm-chessboard'),
    import('cm-chessboard/src/extensions/markers/Markers.js').catch(() => null)
  ]);
  if (!container?.isConnected) return { board: undefined, INPUT_EVENT_TYPE };
  const fen = typeof position === 'function' ? position() : position;
  const extensions = markers
    ? [
        {
          class: markers.Markers,
          props: {
            autoMarkers: markers.MARKER_TYPE.frame,
            sprite: MARKERS_URL
          }
        }
      ]
    : [];
  const board = new Chessboard(container, {
    position: fen,
    orientation,
    assetsUrl: '',
    // cm-chessboard mutates props.style (defaults like cssClass are assigned
    // in), so each mount gets a fresh copy of the frozen contract.
    style: { ...CHESSBOARD_STYLE, pieces: { ...CHESSBOARD_STYLE.pieces } },
    extensions
  });
  // The container carries the accessible name; the injected svg is decorative
  // chrome and must not surface as a second, unnamed img.
  container.querySelector('svg')?.setAttribute('aria-hidden', 'true');
  return { board, INPUT_EVENT_TYPE };
}
