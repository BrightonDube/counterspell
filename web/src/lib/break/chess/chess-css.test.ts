// CH03/CH04/CH06: the board sizes itself from content width AND viewport
// height, the squares/coords/move states speak Steel Night tokens, and the
// skull moves only on the compositor with a reduced-motion collapse.
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(resolve(here, '../../../app.css'), 'utf8');

describe('chess board sizing (CH03)', () => {
  it('bounds the board by content width and viewport height', () => {
    const rule = css.match(/\.break-board\s*\{([^}]*)\}/);
    expect(rule, '.break-board rule').not.toBeNull();
    const body = rule?.[1] ?? '';
    expect(body).toMatch(/min\(/);
    expect(body).toMatch(/dvh|vh/);
    expect(body).toMatch(/%/);
    // Wide screens should get substantially more than the old 448px cap.
    const cap = body.match(/max-inline-size:\s*(\d+)px/);
    expect(cap, 'max-inline-size').not.toBeNull();
    expect(Number(cap?.[1])).toBeGreaterThanOrEqual(560);
  });

  it('floors the board so short viewports never collapse it to zero', () => {
    const rule = css.match(/\.break-board\s*\{([^}]*)\}/);
    const body = rule?.[1] ?? '';
    // 100dvh - 20rem is <=0 under a 320px-tall viewport; a clamp floor keeps
    // squares tappable and lets the page scroll instead.
    expect(body).toMatch(/clamp\(\s*(\d+)px/);
    const floor = body.match(/clamp\(\s*(\d+)px/);
    expect(Number(floor?.[1])).toBeGreaterThanOrEqual(240);
  });
});

describe('steel-night board theme (CH04)', () => {
  it('paints squares, frame and coordinates with semantic tokens', () => {
    for (const sel of ['square.white', 'square.black']) {
      const rule = css.match(
        new RegExp(
          `\\.cm-chessboard\\.steel-night [^}]*\\.${sel.replace('.', '\\.')}\\s*\\{([^}]*)\\}`
        )
      );
      expect(rule, sel).not.toBeNull();
      expect(rule?.[1], sel).toMatch(/fill:\s*var\(--/);
    }
    expect(css).toMatch(
      /\.cm-chessboard\.steel-night\.border-type-frame[^}]*\.border[^}]*fill:\s*var\(--/
    );
    expect(css).toMatch(
      /\.cm-chessboard\.steel-night[^}]*\.coordinate[^}]*fill:\s*var\(--/
    );
  });

  it('themes the move-state markers', () => {
    expect(css).toMatch(
      /\.cm-chessboard\.steel-night[^}]*\.marker-frame[^}]*stroke:\s*var\(--/
    );
    expect(css).toMatch(
      /\.cm-chessboard\.steel-night[^}]*\.marker-dot[^}]*fill:\s*var\(--/
    );
    // The last-move frame is a distinct primary type — it must read as a
    // deliberate highlight, not the ephemeral input frame.
    expect(css).toMatch(
      /\.cm-chessboard\.steel-night[^}]*\.marker-frame-primary[^}]*(stroke|fill):\s*var\(--/
    );
  });
});

describe('skull opponent motion (CH06)', () => {
  it('keeps the hover loop on the compositor with long pauses', () => {
    const keyframes = css.match(/@keyframes skull-hover\s*\{([\s\S]*?)\n\}/);
    expect(keyframes, 'skull-hover keyframes').not.toBeNull();
    const props = [
      ...(keyframes?.[1].matchAll(/^\s+([a-z-]+)\s*:/gm) ?? [])
    ].map((m) => m[1]);
    expect(new Set(props)).toEqual(new Set(['transform']));
    // "Spaced": most of the cycle is still — a long dwell dominates.
    const rule = css.match(/\.skull-sentinel\s*\{([^}]*)\}/);
    expect(rule, '.skull-sentinel').not.toBeNull();
    expect(rule?.[1]).toMatch(/pointer-events:\s*none/);
    expect(rule?.[1]).toMatch(/animation:\s*skull-hover\s+([6-9]|\d\d)s/);
  });
});
