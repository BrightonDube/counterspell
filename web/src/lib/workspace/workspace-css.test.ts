// Checks the workspace costume CSS in app.css (WP06). The ember is atmosphere:
// it must animate only compositor properties, never intercept input, and fall
// under the global reduced-motion collapse.
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(resolve(here, '../../app.css'), 'utf8');

describe('workspace costume CSS (WP06)', () => {
  it('keeps ember-drift on the compositor (transform/opacity only)', () => {
    const keyframes = css.match(/@keyframes ember-drift\s*\{([\s\S]*?)\n\}/);
    expect(keyframes, 'ember-drift keyframes').not.toBeNull();
    const props = [
      ...(keyframes?.[1].matchAll(/^\s+([a-z-]+)\s*:/gm) ?? [])
    ].map((m) => m[1]);
    expect(new Set(props)).toEqual(new Set(['transform', 'opacity']));
  });

  it('renders the ember as a non-interactive animated hairline', () => {
    const rule = css.match(/\.workspace-ember\s*\{([^}]*)\}/);
    expect(rule, '.workspace-ember rule').not.toBeNull();
    expect(rule?.[1]).toMatch(/pointer-events:\s*none/);
    expect(rule?.[1]).toMatch(/animation:\s*ember-drift/);
  });

  it('falls under the global reduced-motion collapse', () => {
    const reduced = css.match(
      /@media \(prefers-reduced-motion: reduce\)\s*\{([\s\S]*?)\n\s*\}/
    );
    expect(reduced, 'global reduced-motion block').not.toBeNull();
    // The collapse targets every element, so the ember needs no own override.
    expect(reduced?.[1]).toMatch(/\*\s*,/);
    expect(reduced?.[1]).toMatch(/animation-duration:\s*0\.01ms\s*!important/);
  });
});
