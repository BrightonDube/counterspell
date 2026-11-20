import { describe, expect, it } from 'vitest';
import { checkPress, level, newRound } from './memory.js';

describe('memory game', () => {
  it('grows the sequence with an injected rng', () => {
    const seq = newRound([1, 2], () => 0.99);
    expect(seq).toEqual([1, 2, 8]);
    expect(level(seq)).toBe(1);
  });

  it('checks presses', () => {
    const seq = [0, 4, 8];
    expect(checkPress(seq, [0], 0)).toBe('continue');
    expect(checkPress(seq, [0, 4], 1)).toBe('continue');
    expect(checkPress(seq, [0, 4, 8], 2)).toBe('round_done');
    expect(checkPress(seq, [0, 5], 1)).toBe('wrong');
  });
});
