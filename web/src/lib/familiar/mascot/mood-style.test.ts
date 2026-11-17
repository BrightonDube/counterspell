import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import type { Mood } from '../types';
import { LAYER_IDS } from './mood-style';
import { glowCss, moodStyle, textTokenFor } from './mood-style';

const MOODS: Mood[] = [1, 2, 3, 4, 5];
const HERE = dirname(fileURLToPath(import.meta.url));

describe('moodStyle', () => {
  it('names each mood, from furious to laughing', () => {
    expect(MOODS.map((mood) => moodStyle(mood).name)).toEqual([
      'furious',
      'angry',
      'uneasy',
      'pleased',
      'laughing'
    ]);
  });

  it('uses the icon for each mood, so colour is never the only signal', () => {
    expect(MOODS.map((mood) => moodStyle(mood).icon)).toEqual([
      'cross',
      'warning',
      'caution',
      'smile',
      'sparkle'
    ]);
  });

  // Breaks if a mood points at a layer the approved art does not have, so that mood shows no painting.
  it('shows each mood with its own painted image layer, which the artwork contract requires', () => {
    for (const mood of MOODS) {
      const style = moodStyle(mood);
      expect(style.layer).toBe(`fam-mood-${mood}`);
      expect(LAYER_IDS).toContain(style.layer);
    }
  });

  it('builds each glow from palette tokens and the design mix percentages', () => {
    expect(moodStyle(1).glow).toEqual({ base: '--danger-fill' });
    expect(moodStyle(2).glow).toEqual({
      base: '--danger-fill',
      mixWith: '--accent-candle',
      mixPercent: 55
    });
    expect(moodStyle(3).glow).toEqual({
      base: '--accent-candle',
      mixWith: '--bg',
      mixPercent: 55
    });
    expect(moodStyle(4).glow).toEqual({ base: '--accent-candle' });
    expect(moodStyle(5).glow).toEqual({ base: '--accent-candle-glow' });
  });
});

describe('textTokenFor', () => {
  it('uses the danger text colour for a failed run', () => {
    expect(textTokenFor(true)).toBe('--danger');
  });

  it('never uses the danger text colour for a passing run, whatever the mood', () => {
    for (const mood of MOODS) {
      // The mood does not change the text colour of a passing run.
      expect(textTokenFor(false)).not.toBe('--danger');
      expect(moodStyle(mood).glow.base).not.toBe('--success');
    }
  });
});

describe('no literal colours', () => {
  it('keeps every hex colour out of the mascot modules', () => {
    for (const file of ['mood-style.ts', 'plan.ts']) {
      const source = readFileSync(join(HERE, file), 'utf8');
      expect(source).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
    }
  });
});

describe('glowCss', () => {
  // Breaks if a mixed glow drops its second token, so the ember and dim moods look like plain fills.
  it('mixes two palette tokens with colour-mix and never uses a hex value', () => {
    expect(
      glowCss({
        base: '--danger-fill',
        mixWith: '--accent-candle',
        mixPercent: 55
      })
    ).toBe('color-mix(in srgb, var(--danger-fill) 55%, var(--accent-candle))');
  });

  // Breaks if a plain glow is wrapped in a mix, which would add an unintended second colour.
  it('uses a single palette token when the recipe mixes nothing', () => {
    expect(glowCss({ base: '--accent-candle' })).toBe('var(--accent-candle)');
  });
});
