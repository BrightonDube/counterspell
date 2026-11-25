// CH02–CH05: one shared presentation contract for bot and puzzle boards —
// the Steel Night cssClass, framed coordinates, the original piece sprite and
// bounded animation. Both routes must consume this object unchanged.
import { describe, expect, it, vi } from 'vitest';

vi.mock('#lib/chess/pieces-steel-night.svg?url', () => ({
  default: '/assets/pieces-steel-night.svg'
}));
vi.mock('cm-chessboard/assets/extensions/markers/markers.svg?url', () => ({
  default: '/assets/markers.svg'
}));

import { CHESSBOARD_STYLE } from './theme.js';

describe('shared chessboard presentation (CH02–CH05)', () => {
  it('uses the steel-night theme class with framed coordinates', () => {
    expect(CHESSBOARD_STYLE.cssClass).toBe('steel-night');
    expect(CHESSBOARD_STYLE.borderType).toBe('frame');
    expect(CHESSBOARD_STYLE.showCoordinates).toBe(true);
    expect(CHESSBOARD_STYLE.aspectRatio).toBe(1);
  });

  it('points at the original 2.5D piece sprite at tile size 40', () => {
    expect(CHESSBOARD_STYLE.pieces.tileSize).toBe(40);
    expect(CHESSBOARD_STYLE.pieces.file).toMatch(/steel-night.*\.svg$/);
  });

  it('keeps piece animation bounded and off the critical path', () => {
    expect(CHESSBOARD_STYLE.animationDuration).toBeGreaterThan(0);
    expect(CHESSBOARD_STYLE.animationDuration).toBeLessThanOrEqual(300);
  });
});
