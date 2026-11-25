// Shared cm-chessboard presentation for the bot and puzzle routes (CH03–CH05):
// one Steel Night theme class, the original piece sprite, framed coordinates
// and the last-move marker — so both boards look and behave identically.
import piecesSpriteUrl from '#lib/break/chess/pieces-steel-night.svg?url';
import markersSpriteUrl from 'cm-chessboard/assets/extensions/markers/markers.svg?url';

/** Style contract shared by every Counterspell board. */
export const CHESSBOARD_STYLE = Object.freeze({
  cssClass: 'steel-night',
  showCoordinates: true,
  borderType: 'frame',
  aspectRatio: 1,
  pieces: { tileSize: 40, file: piecesSpriteUrl },
  animationDuration: 300
});

/**
 * Mounts a themed board in `container`. cm-chessboard ships `main: null`, so
 * both imports stay lazy — the routes remain server-renderable. The Markers
 * extension adds the auto last-move frame; its sprite path is passed absolute
 * (bundled), with `assetsUrl` emptied so the absolute URL survives.
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
            sprite: markersSpriteUrl
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
