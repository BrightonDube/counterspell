import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { TILE_ART, checkPress, level, newRound } from './memory.js';

const here = dirname(fileURLToPath(import.meta.url));

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

describe('TILE_ART (LR03)', () => {
  it('assigns every tile one stable themed image', () => {
    // Deterministic: tile i always maps to the same cached URL.
    expect(TILE_ART).toHaveLength(9);
    expect(new Set(TILE_ART).size).toBe(9);
    for (const url of TILE_ART) {
      expect(url).toMatch(/^\/art\/objects\/[a-z-]+\.webp$/);
    }
  });

  it('points at files that actually ship', () => {
    for (const url of TILE_ART) {
      const onDisk = resolve(here, '../../../static', '.' + url);
      expect(existsSync(onDisk), url).toBe(true);
    }
  });
});
