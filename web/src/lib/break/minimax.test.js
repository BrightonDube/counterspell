import { describe, expect, it } from 'vitest';
import { Chess } from 'chess.js';
import { chooseMove, seededRng } from './minimax.js';

const POSITIONS = [
  'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
  'r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3',
  '8/8/4k3/8/8/4K3/4P3/8 w - - 0 1',
  '6k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1',
];

describe('chooseMove', () => {
  it('takes a free mate in one', () => {
    const fen = '6k1/pp3ppp/8/8/8/8/PP3PPP/4R1K1 w - - 0 1';
    const { san, uci } = chooseMove(fen, { rng: seededRng('mate') });
    if (!uci) throw new Error('expected a move');
    const game = new Chess(fen);
    const played = game.move({ from: uci.slice(0, 2), to: uci.slice(2, 4) });
    expect(played.san).toBe(san);
    expect(game.isCheckmate()).toBe(true);
  });

  it('returns a legal move in every listed position', () => {
    for (const fen of POSITIONS) {
      const { uci } = chooseMove(fen, { rng: seededRng('x') });
      if (!uci) throw new Error('expected a move');
      const game = new Chess(fen);
      const played = game.move({ from: uci.slice(0, 2), to: uci.slice(2, 4) });
      expect(played).toBeTruthy();
    }
  });

  it('is repeatable with the same seed', () => {
    const fen = POSITIONS[1];
    const a = chooseMove(fen, { rng: seededRng('same-seed') });
    const b = chooseMove(fen, { rng: seededRng('same-seed') });
    expect(a.uci).toBe(b.uci);
    expect(a.nodes).toBeGreaterThan(0);
  });
});
