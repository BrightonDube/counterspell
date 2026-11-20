import { describe, expect, it } from 'vitest';
import { newPuzzle, tryMove } from './puzzle.js';

// Black to move plays Re8 (auto), then White (user) answers Rxe8#.
const FEN = '5rk1/pp3ppp/8/8/8/8/PP3PPP/4R1K1 b - - 0 1';
const SOLUTION = ['f8e8', 'e1e8'];

describe('puzzle state machine', () => {
  it('plays the opponent first move automatically', () => {
    const state = newPuzzle(
      'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
      ['e2e4', 'e7e5']
    );
    expect(state.index).toBe(1);
    expect(state.game.turn()).toBe('b');
  });

  it('accepts the solution and marks solved', () => {
    const state = newPuzzle(FEN, SOLUTION);
    const { state: after, feedback } = tryMove(state, 'e1e8');
    expect(feedback).toBe('Solved.');
    expect(after.status).toBe('solved');
    expect(after.mistakes).toBe(0);
  });

  it('counts a wrong move without advancing', () => {
    const state = newPuzzle(FEN, SOLUTION);
    const { state: after, feedback } = tryMove(state, 'e1e2');
    expect(feedback).toBe('Not the solution. Try again.');
    expect(after.mistakes).toBe(1);
    expect(after.status).toBe('playing');
  });

  it('rejects illegal input without counting a mistake', () => {
    const state = newPuzzle(FEN, SOLUTION);
    const { state: after, feedback } = tryMove(state, 'e1e9');
    expect(feedback).toBe('Illegal move.');
    expect(after.mistakes).toBe(0);
  });
});
