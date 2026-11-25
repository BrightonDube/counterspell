// CH05: the piece set is ONE original lightweight SVG sprite — all twelve
// piece ids plus the promotion crown, each drawn at the 40px tile the board
// expects, carrying the 2.5D vocabulary (outline, highlight, soft shadow),
// with no raster or script payloads.
import { readFileSync, statSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
// The sprite ships as a same-origin static file (cm-chessboard XHRs it).
const file = resolve(here, '../../../../static/break/pieces-steel-night.svg');
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

  // CPX-1: premium dimensional shading comes from shared gradients —
  // carved ivory for white, obsidian for black — not flat fills.
  it('shades white pieces with an ivory gradient, black with obsidian', () => {
    expect(svg).toMatch(/<linearGradient[^>]*id="ivory"/);
    expect(svg).toMatch(/<linearGradient[^>]*id="obsidian"/);
    for (const id of PIECE_IDS) {
      const body = pieceBody(id);
      const want = id.startsWith('w') ? 'ivory' : 'obsidian';
      expect(body, `${id} body fill`).toMatch(
        new RegExp(`fill="url\\(#${want}`)
      );
    }
  });

  // CPX-1: black pieces carry bright neutral/steel edging so they read
  // on the dark squares; gold accents stay on the white side only.
  it('edges black pieces in steel, keeps candle-gold on white only', () => {
    for (const id of PIECE_IDS) {
      const body = pieceBody(id);
      const group = svg.match(new RegExp(`<g[^>]*id="${id}"[^>]*>`))?.[0] ?? '';
      if (id.startsWith('b')) {
        expect(group, `${id} steel edge`).toMatch(/stroke="#a8bacd"/);
        expect(body, `${id} no gold fill`).not.toMatch(/url\(#gold/);
      } else {
        expect(group, `${id} dark outline`).toMatch(/stroke="#0d1214"/);
      }
    }
  });

  // CPX-1: settled artwork, outlines and shadows stay inside the tile
  // with visible padding — every absolute coordinate in [0.5, 39.5] and
  // the painted span stays under 90% of the 40px tile.
  it('keeps every piece inside the tile with visible padding', () => {
    const MAX_EXTENT = 36; // 90% of the 40px tile
    for (const id of [...PIECE_IDS, 'crown']) {
      const body = pieceBody(id);
      const xs: number[] = [];
      const ys: number[] = [];
      // Ellipse/circle painted extents: cx±rx, cy±ry must stay in-tile.
      for (const m of body.matchAll(/<(ellipse|circle)\b[^>]*>/g)) {
        const cx = Number(m[0].match(/\scx="([\d.-]+)"/)?.[1] ?? NaN);
        const cy = Number(m[0].match(/\scy="([\d.-]+)"/)?.[1] ?? NaN);
        const rx = Number(m[0].match(/\sr(?:x)?="([\d.-]+)"/)?.[1] ?? 0);
        const ry = Number(m[0].match(/\sr(?:y)?="([\d.-]+)"/)?.[1] ?? rx);
        if (Number.isNaN(cx) || Number.isNaN(cy)) continue;
        expect(
          cx - rx,
          `${id} circle/ellipse left edge`
        ).toBeGreaterThanOrEqual(0.5);
        expect(cx + rx, `${id} circle/ellipse right edge`).toBeLessThanOrEqual(
          39.5
        );
        expect(cy - ry, `${id} circle/ellipse top edge`).toBeGreaterThanOrEqual(
          0.5
        );
        expect(cy + ry, `${id} circle/ellipse bottom edge`).toBeLessThanOrEqual(
          39.5
        );
        xs.push(cx - rx, cx + rx);
        ys.push(cy - ry, cy + ry);
      }
      // Plain x/y attributes (word-bounded so opacity/ry don't leak in).
      for (const m of body.matchAll(/[\s"](x|cx|y|cy)="([\d.-]+)"/g)) {
        if (m[1] === 'x' || m[1] === 'cx') xs.push(Number(m[2]));
        else ys.push(Number(m[2]));
      }
      for (const d of body.matchAll(/\sd="([^"]+)"/g)) {
        const { absX, absY } = absoluteCoords(d[1]);
        xs.push(...absX);
        ys.push(...absY);
      }
      expect(xs.length, `${id} has coordinates`).toBeGreaterThan(0);
      for (const x of xs) {
        expect(x, `${id} x=${x}`).toBeGreaterThanOrEqual(0.5);
        expect(x, `${id} x=${x}`).toBeLessThanOrEqual(39.5);
      }
      for (const y of ys) {
        expect(y, `${id} y=${y}`).toBeGreaterThanOrEqual(0.5);
        expect(y, `${id} y=${y}`).toBeLessThanOrEqual(39.5);
      }
      const spanX = Math.max(...xs) - Math.min(...xs);
      const spanY = Math.max(...ys) - Math.min(...ys);
      expect(spanX, `${id} x-extent ${spanX}`).toBeLessThanOrEqual(MAX_EXTENT);
      expect(spanY, `${id} y-extent ${spanY}`).toBeLessThanOrEqual(MAX_EXTENT);
    }
  });
});

function pieceBody(id: string): string {
  const piece = svg.match(
    new RegExp(`<g[^>]*id="${id}"[^>]*>([\\s\\S]*?)</g>`)
  );
  expect(piece, id).not.toBeNull();
  return piece?.[1] ?? '';
}

// Extracts absolute coordinates from an SVG path `d` string. Absolute
// commands contribute positions; relative commands are skipped (their
// deltas are small and cannot place painted pixels outside the tile on
// their own — they are anchored to an absolute start point).
function absoluteCoords(d: string): { absX: number[]; absY: number[] } {
  const arity: Record<string, number> = {
    M: 2,
    L: 2,
    C: 6,
    S: 4,
    Q: 4,
    T: 2,
    A: 7,
    H: 1,
    V: 1,
    Z: 0
  };
  const absX: number[] = [];
  const absY: number[] = [];
  const tokens = d.match(/[a-zA-Z]|-?\.?\d[\d.]*|-?\.\d+/g) ?? [];
  let cmd = '';
  let i = 0;
  while (i < tokens.length) {
    if (/[a-zA-Z]/.test(tokens[i])) {
      cmd = tokens[i];
      i += 1;
      continue;
    }
    const upper = cmd.toUpperCase();
    const n = arity[upper];
    if (n === undefined || n === 0) {
      i += 1;
      continue;
    }
    const args = tokens.slice(i, i + n).map(Number);
    i += n;
    if (cmd === upper) {
      // Absolute command: collect coordinate positions.
      if (upper === 'H') absX.push(args[0]);
      else if (upper === 'V') absY.push(args[0]);
      else if (upper === 'A') {
        absX.push(args[5]);
        absY.push(args[6]);
      } else {
        for (let k = 0; k + 1 < args.length; k += 2) {
          absX.push(args[k]);
          absY.push(args[k + 1]);
        }
      }
    }
  }
  return { absX, absY };
}
