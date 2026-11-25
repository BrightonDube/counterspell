// Shared cm-chessboard presentation for the bot and puzzle routes (CH03–CH05):
// one Steel Night theme class, the original piece sprite, framed coordinates
// and the last-move marker — so both boards look and behave identically.
//
// Sprites are literal same-origin paths under web/static/break/: cm-chessboard
// fetches them with XMLHttpRequest, so a `?url` import that Vite inlines to a
// `data:` URL would violate the deployed `connect-src 'self'` policy (the
// marker sprite is below Vite's 4 kB inlining threshold).
const PIECES_URL = '/break/pieces-steel-night.svg';
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
 * Mounts a themed board in `container`. cm-chessboard ships `main: null`, so
 * both imports stay lazy — the routes remain server-renderable. The Markers
 * extension adds the selection frames on top.
 * @param {HTMLElement | undefined} container
 * @param {string} position FEN to show
 * @param {'w' | 'b'} [orientation]
 */
export async function mountThemedBoard(container, position, orientation = 'w') {
  const [{ Chessboard, INPUT_EVENT_TYPE }, markers] = await Promise.all([
    import('cm-chessboard'),
    import('cm-chessboard/src/extensions/markers/Markers.js').catch(() => null)
  ]);
  if (!container?.isConnected) return { board: undefined, INPUT_EVENT_TYPE };
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
    position,
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
