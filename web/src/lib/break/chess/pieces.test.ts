// CH05: the piece set is ONE original lightweight SVG sprite — all twelve
// piece ids plus the promotion crown, each drawn at the 40px tile the board
// expects, carrying the 2.5D vocabulary (outline, highlight, soft shadow),
// with no raster or script payloads.
import { readFileSync, statSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
const file = resolve(here, 'pieces-steel-night.svg');
const svg = readFileSync(file, 'utf8');

const PIECE_IDS = [
  'wp',
  'wn',
  'wb',
  'wr',
  'wq',
  'wk',
  'bp',
  'bn',
  'bb',
  'br',
  'bq',
  'bk'
];

describe('steel-night piece sprite (CH05)', () => {
  it('is a single 40px-tile SVG sprite with every piece id', () => {
    expect(svg).toMatch(/<svg[^>]*viewBox="0 0 40 40"/);
    for (const id of [...PIECE_IDS, 'crown']) {
      expect(svg, id).toMatch(new RegExp(`<g[^>]*id="${id}"`));
    }
  });

  it('gives every piece the 2.5D vocabulary: outline, highlight, shadow', () => {
    for (const id of PIECE_IDS) {
      const piece = svg.match(
        new RegExp(`<g[^>]*id="${id}"[^>]*>([\\s\\S]*?)</g>`)
      );
      expect(piece, id).not.toBeNull();
      const body = piece?.[1] ?? '';
      expect(body, `${id} outline`).toMatch(/stroke=/);
      expect(body, `${id} highlight`).toMatch(/class="hi"/);
      expect(body, `${id} shadow`).toMatch(/class="sh"/);
    }
  });

  it('stays lightweight and vector-only', () => {
    expect(svg).not.toMatch(/<image|<script|<foreignObject/);
    expect(statSync(file).size).toBeLessThan(40_000);
  });
});
