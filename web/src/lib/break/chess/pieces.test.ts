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
      expect(svg, id).toContain(`<g id="${id}"`);
    }
  });

  it('gives every piece the 2.5D vocabulary: outline, highlight, shadow', () => {
    for (const id of PIECE_IDS) {
      const body = pieceBody(id);
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
      expect(body, `${id} body fill`).toContain(`fill="url(#${want}`);
    }
  });

  // CPX-1: black pieces carry bright neutral/steel edging so they read
  // on the dark squares; gold accents stay on the white side only.
  it('edges black pieces in steel, keeps candle-gold on white only', () => {
    for (const id of PIECE_IDS) {
      const body = pieceBody(id);
      const group = pieceTag(id);
      if (id.startsWith('b')) {
        expect(group, `${id} steel edge`).toMatch(/stroke="#a8bacd"/);
        expect(body, `${id} no gold fill`).not.toMatch(/url\(#gold/);
      } else {
        expect(group, `${id} dark outline`).toMatch(/stroke="#0d1214"/);
      }
    }
  });

  // CPX-1: settled artwork, outlines and shadows stay inside the tile
  // with visible padding — painted edges (geometry plus half the widest
  // stroke) remain in [0.5, 39.5] and the span stays under 90% of the
  // 40px tile.
  it('keeps every piece inside the tile with visible padding', () => {
    const PAD = 0.5;
    const MAX_EXTENT = 36; // 90% of the 40px tile
    for (const id of [...PIECE_IDS, 'crown']) {
      const body = pieceBody(id);
      // Painted ink extends half a stroke width past the geometry.
      const strokeHalf =
        Math.max(
          0,
          ...Array.from(body.matchAll(/stroke-width="([\d.]+)"/g), (m) =>
            Number(m[1])
          )
        ) / 2;
      const xs: number[] = [];
      const ys: number[] = [];
      // Ellipse/circle painted extents: cx±rx, cy±ry must stay in-tile.
      for (const m of body.matchAll(/<(ellipse|circle)\b[^>]*>/g)) {
        const cx = Number(m[0].match(/\scx="([\d.-]+)"/)?.[1] ?? NaN);
        const cy = Number(m[0].match(/\scy="([\d.-]+)"/)?.[1] ?? NaN);
        const rx = Number(m[0].match(/\sr(?:x)?="([\d.-]+)"/)?.[1] ?? 0);
        const ry = Number(m[0].match(/\sr(?:y)?="([\d.-]+)"/)?.[1] ?? rx);
        if (Number.isNaN(cx) || Number.isNaN(cy)) continue;
        const margin = m[0].includes('stroke="none"') ? 0 : strokeHalf;
        expect(
          cx - rx - margin,
          `${id} circle/ellipse left edge`
        ).toBeGreaterThanOrEqual(PAD);
        expect(
          cx + rx + margin,
          `${id} circle/ellipse right edge`
        ).toBeLessThanOrEqual(39.5);
        expect(
          cy - ry - margin,
          `${id} circle/ellipse top edge`
        ).toBeGreaterThanOrEqual(PAD);
        expect(
          cy + ry + margin,
          `${id} circle/ellipse bottom edge`
        ).toBeLessThanOrEqual(39.5);
        xs.push(cx - rx - margin, cx + rx + margin);
        ys.push(cy - ry - margin, cy + ry + margin);
      }
      // Paths: resolve relative segments to absolute positions and add
      // the stroke margin for elements that are actually stroked.
      for (const el of body.matchAll(/<path\b[^>]*>/g)) {
        const d = el[0].match(/\sd="([^"]+)"/)?.[1];
        if (!d) continue;
        const margin = el[0].includes('stroke="none"') ? 0 : strokeHalf;
        const { xs: px, ys: py } = paintedCoords(d);
        xs.push(...px.map((x) => x - margin), ...px.map((x) => x + margin));
        ys.push(...py.map((y) => y - margin), ...py.map((y) => y + margin));
      }
      expect(xs.length, `${id} has coordinates`).toBeGreaterThan(0);
      for (const x of xs) {
        expect(x, `${id} x=${x}`).toBeGreaterThanOrEqual(PAD);
        expect(x, `${id} x=${x}`).toBeLessThanOrEqual(39.5);
      }
      for (const y of ys) {
        expect(y, `${id} y=${y}`).toBeGreaterThanOrEqual(PAD);
        expect(y, `${id} y=${y}`).toBeLessThanOrEqual(39.5);
      }
      const spanX = Math.max(...xs) - Math.min(...xs);
      const spanY = Math.max(...ys) - Math.min(...ys);
      expect(spanX, `${id} x-extent ${spanX}`).toBeLessThanOrEqual(
        MAX_EXTENT + strokeHalf * 2
      );
      expect(spanY, `${id} y-extent ${spanY}`).toBeLessThanOrEqual(
        MAX_EXTENT + strokeHalf * 2
      );
    }
  });
});

// String-sliced group extraction — no dynamic RegExp (ReDoS guard).
function pieceBody(id: string): string {
  const at = svg.indexOf(`id="${id}"`);
  expect(at, id).toBeGreaterThanOrEqual(0);
  const openEnd = svg.indexOf('>', at) + 1;
  const close = svg.indexOf('</g>', openEnd);
  expect(close, id).toBeGreaterThan(openEnd);
  return svg.slice(openEnd, close);
}

function pieceTag(id: string): string {
  const at = svg.indexOf(`id="${id}"`);
  expect(at, id).toBeGreaterThanOrEqual(0);
  const open = svg.lastIndexOf('<g', at);
  expect(open, id).toBeGreaterThanOrEqual(0);
  return svg.slice(open, svg.indexOf('>', open) + 1);
}

// Tracks the cursor through an SVG path `d` string so relative commands
// resolve to absolute positions. Bezier control points are recorded too
// (the painted curve stays inside its control-point hull), and arc
// endpoints are widened by their radii as a conservative envelope — the
// collected set always over-approximates the painted geometry.
function paintedCoords(d: string): { xs: number[]; ys: number[] } {
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
  const xs: number[] = [];
  const ys: number[] = [];
  const tokens = d.match(/[a-zA-Z]|-?(?:\d+\.?\d*|\.\d+)/g) ?? [];
  let cmd = '';
  let i = 0;
  let cx = 0;
  let cy = 0;
  let startX = 0;
  let startY = 0;
  while (i < tokens.length) {
    if (/[a-zA-Z]/.test(tokens[i])) {
      cmd = tokens[i];
      i += 1;
      if (cmd.toUpperCase() === 'Z') {
        cx = startX;
        cy = startY;
      }
      continue;
    }
    const upper = cmd.toUpperCase();
    const n = arity[upper];
    if (n === undefined || n === 0) {
      i += 1;
      continue;
    }
    const rel = cmd !== upper;
    const args = tokens.slice(i, i + n).map(Number);
    i += n;
    const px = (v: number) => (rel ? cx + v : v);
    const py = (v: number) => (rel ? cy + v : v);
    if (upper === 'H') {
      cx = px(args[0]);
      xs.push(cx);
    } else if (upper === 'V') {
      cy = py(args[0]);
      ys.push(cy);
    } else if (upper === 'A') {
      cx = px(args[5]);
      cy = py(args[6]);
      // Conservative arc envelope: endpoint expanded by the radii.
      xs.push(cx - args[0], cx + args[0]);
      ys.push(cy - args[1], cy + args[1]);
    } else {
      for (let k = 0; k + 1 < args.length; k += 2) {
        xs.push(px(args[k]));
        ys.push(py(args[k + 1]));
      }
      cx = px(args[args.length - 2]);
      cy = py(args[args.length - 1]);
    }
    if (upper === 'M') {
      startX = cx;
      startY = cy;
    }
  }
  return { xs, ys };
}
