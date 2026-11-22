// @vitest-environment node
// The small art variants the landing page offers in `srcset` (R24). Read from the real files.
import { readFileSync, statSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const ART = new URL('../../../static/art/', import.meta.url);

/** Reads the pixel size from a WebP header (lossy, lossless or extended). */
function webpSize(path: string): { width: number; height: number } {
  const bytes = readFileSync(new URL(path, ART));
  expect(bytes.toString('ascii', 0, 4)).toBe('RIFF');
  expect(bytes.toString('ascii', 8, 12)).toBe('WEBP');
  const kind = bytes.toString('ascii', 12, 16);
  if (kind === 'VP8X') {
    return {
      width: 1 + bytes.readUIntLE(24, 3),
      height: 1 + bytes.readUIntLE(27, 3)
    };
  }
  if (kind === 'VP8 ') {
    return {
      width: bytes.readUInt16LE(26) & 0x3fff,
      height: bytes.readUInt16LE(28) & 0x3fff
    };
  }
  const bits = bytes.readUInt32LE(21);
  return { width: (bits & 0x3fff) + 1, height: ((bits >> 14) & 0x3fff) + 1 };
}

const VARIANTS: [string, number][] = [
  ['objects/crypt-key-128.webp', 128],
  ['objects/crystal-ball-128.webp', 128],
  ['objects/brain-jar-128.webp', 128],
  ['objects/candle-flame-96.webp', 96],
  ['objects/candle-flame-192.webp', 192],
  ['familiar/jack-o-lantern-orange-96.webp', 96],
  ['familiar/jack-o-lantern-orange-192.webp', 192]
];

describe('landing art variants', () => {
  it.each(VARIANTS)('%s is %i px square and under 12 KB', (path, side) => {
    expect(webpSize(path)).toEqual({ width: side, height: side });
    expect(statSync(new URL(path, ART)).size).toBeLessThan(12 * 1024);
  });

  it('keeps every original 512 px file in place for the other pages', () => {
    for (const path of [
      'objects/crypt-key.webp',
      'objects/crystal-ball.webp',
      'objects/brain-jar.webp',
      'objects/candle-flame.webp',
      'familiar/jack-o-lantern-orange.webp'
    ]) {
      expect(webpSize(path)).toEqual({ width: 512, height: 512 });
    }
  });
});
