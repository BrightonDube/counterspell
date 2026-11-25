// CH02–CH05: one shared presentation contract for bot and puzzle boards —
// the Steel Night cssClass, framed coordinates, the original piece sprite and
// bounded animation. Both routes must consume this object unchanged.
import { describe, expect, it, vi } from 'vitest';

vi.mock('cm-chessboard', () => ({
  Chessboard: class {
    constructor(_el: HTMLElement, props: Record<string, unknown>) {
      mounted.props = props;
    }
    setPosition() {}
  },
  INPUT_EVENT_TYPE: { validateMoveInput: 'validateMoveInput' }
}));
vi.mock('cm-chessboard/src/extensions/markers/Markers.js', () => ({
  Markers: function Markers() {},
  MARKER_TYPE: { frame: { class: 'marker-frame', slice: 'markerFrame' } }
}));

const mounted = vi.hoisted(() => ({
  props: undefined as Record<string, unknown> | undefined
}));

import {
  CHESSBOARD_STYLE,
  LAST_MOVE_MARKER,
  clearLastMove,
  mountThemedBoard,
  setBoardOrientation,
  showLastMove
} from './theme.js';

describe('shared chessboard presentation (CH02–CH05)', () => {
  it('uses the steel-night theme class with framed coordinates', () => {
    expect(CHESSBOARD_STYLE.cssClass).toBe('steel-night');
    expect(CHESSBOARD_STYLE.borderType).toBe('frame');
    expect(CHESSBOARD_STYLE.showCoordinates).toBe(true);
    expect(CHESSBOARD_STYLE.aspectRatio).toBe(1);
  });

  it('points at the original 2.5D piece sprite at tile size 40', () => {
    expect(CHESSBOARD_STYLE.pieces.tileSize).toBe(40);
    // Same-origin file URL — cm-chessboard XHRs sprites, so a bundled
    // `?url` import that Vite inlines to `data:` is blocked by connect-src.
    expect(CHESSBOARD_STYLE.pieces.file).toBe('/break/pieces-steel-night.svg');
  });

  it('keeps piece animation bounded and off the critical path', () => {
    expect(CHESSBOARD_STYLE.animationDuration).toBeGreaterThan(0);
    expect(CHESSBOARD_STYLE.animationDuration).toBeLessThanOrEqual(300);
  });

  it('mounts with a same-origin marker sprite (no data: URL)', async () => {
    const container = document.createElement('div');
    document.body.appendChild(container);
    await mountThemedBoard(container, 'startpos');
    const ext = (
      mounted.props?.extensions as { props?: { sprite?: string } }[]
    )?.[0];
    expect(ext?.props?.sprite).toBe('/break/markers.svg');
    container.remove();
  });

  it('evaluates a position getter after the lazy imports resolve', async () => {
    const container = document.createElement('div');
    document.body.appendChild(container);
    // A game can advance while the cm-chessboard imports are in flight —
    // the getter must be read at mount time, not captured by the caller.
    let fen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1';
    const pending = mountThemedBoard(container, () => fen);
    fen = 'rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2';
    await pending;
    expect(mounted.props?.position).toContain(' b ');
    container.remove();
  });
});

describe('last-move marker (CH04)', () => {
  const fakeBoard = () => ({
    addMarker: vi.fn(),
    removeMarkers: vi.fn()
  });

  it('frames both squares of the last applied move', () => {
    const board = fakeBoard();
    showLastMove(board, 'e2', 'e4');
    expect(board.removeMarkers).toHaveBeenCalledWith(LAST_MOVE_MARKER);
    expect(board.addMarker).toHaveBeenNthCalledWith(1, LAST_MOVE_MARKER, 'e2');
    expect(board.addMarker).toHaveBeenNthCalledWith(2, LAST_MOVE_MARKER, 'e4');
    // Distinct from the input-selection frame the extension clears itself.
    expect(LAST_MOVE_MARKER.class).toBe('marker-frame-primary');
  });

  it('clears the frame on a new game', () => {
    const board = fakeBoard();
    clearLastMove(board);
    expect(board.removeMarkers).toHaveBeenCalledWith(LAST_MOVE_MARKER);
  });

  it('tolerates a missing board', () => {
    expect(() => showLastMove(undefined, 'e2', 'e4')).not.toThrow();
    expect(() => clearLastMove(undefined)).not.toThrow();
  });
});

describe('board orientation (CPX05)', () => {
  // cm-chessboard drops a setOrientation call while a turn is queued; the
  // shared helper serializes requests so the newest side still applies.
  const fakeBoard = () => {
    const calls: string[] = [];
    const resolvers: (() => void)[] = [];
    const board = {
      orientation: 'w',
      calls,
      resolvers,
      getOrientation() {
        return board.orientation;
      },
      setOrientation(color: string) {
        calls.push(color);
        return new Promise<void>((resolve) => {
          resolvers.push(() => {
            board.orientation = color;
            resolve();
          });
        });
      }
    };
    return board;
  };
  const flush = async () => {
    await Promise.resolve();
    await Promise.resolve();
    await new Promise((resolve) => setTimeout(resolve, 0));
  };

  it('skips a redundant turn when the side already faces the player', () => {
    const board = fakeBoard();
    setBoardOrientation(board as never, 'w');
    expect(board.calls).toEqual([]);
  });

  it('re-issues the newest side after a pending turn instead of dropping it', async () => {
    const board = fakeBoard();
    setBoardOrientation(board as never, 'b');
    setBoardOrientation(board as never, 'w');
    expect(board.calls).toEqual(['b']);
    board.resolvers[0]?.();
    await flush();
    expect(board.calls).toEqual(['b', 'w']);
    board.resolvers[1]?.();
    await flush();
    expect(board.orientation).toBe('w');
  });

  it('applies the queued side when it differs after the first turn', async () => {
    const board = fakeBoard();
    setBoardOrientation(board as never, 'b');
    setBoardOrientation(board as never, 'b');
    expect(board.calls).toEqual(['b']);
    board.resolvers[0]?.();
    await flush();
    // The queued 'b' re-checks: the board is already 'b', so nothing re-issues.
    expect(board.calls).toEqual(['b']);
  });

  it('tolerates a missing board and a board without setOrientation', () => {
    expect(() => setBoardOrientation(undefined, 'b')).not.toThrow();
    expect(() => setBoardOrientation({} as never, 'b')).not.toThrow();
  });
});
