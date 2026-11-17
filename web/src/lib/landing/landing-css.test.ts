// Checks the landing CSS fallbacks in app.css (T11). Reduced motion must remove transitions,
// and the landing block must add no raw colour values.
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(resolve(here, '../../app.css'), 'utf8');

/** The landing block, from its marker comment to the end of the file. */
function landingBlock(): string {
  const start = css.indexOf('/* landing: begin */');
  expect(start, 'landing block marker').toBeGreaterThanOrEqual(0);
  return css.slice(start);
}

describe('landing CSS fallbacks (T11)', () => {
  it('turns CTA transitions off under reduced motion', () => {
    const block = landingBlock();
    const reduced = block.match(
      /@media \(prefers-reduced-motion: reduce\)\s*\{([\s\S]*?)\n\}/
    );
    expect(reduced, 'reduced-motion media block').not.toBeNull();
    expect(reduced?.[1]).toMatch(/\.cta\s*\{[^}]*transition-duration:\s*0ms/);
  });

  it('adds no raw colour values in the landing block (tokens only)', () => {
    const block = landingBlock();
    expect(block).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
    expect(block).not.toMatch(/\b(rgb|hsl)a?\(/);
  });
});
