// LR04/LR05: the reveal transition is an opacity crossfade — cheap, and it
// collapses to an instant state under the global reduced-motion rule. No
// spatial flipping (rotate/translate/scale) is allowed anywhere in the tile
// styling, per the reduced-motion requirement.
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const here = dirname(fileURLToPath(import.meta.url));
const src = readFileSync(resolve(here, '+page.svelte'), 'utf8');
const style = src.match(/<style>([\s\S]*?)<\/style>/)?.[1] ?? '';

describe('lantern recall reveal CSS (LR04–LR05)', () => {
  it('fades the tile art with an opacity-only transition', () => {
    const rule = style.match(/\.tile-art\s*\{([^}]*)\}/);
    expect(rule, '.tile-art rule').not.toBeNull();
    expect(rule?.[1]).toMatch(/opacity:\s*0/);
    expect(rule?.[1]).toMatch(/transition:\s*opacity/);
    expect(rule?.[1]).toMatch(/pointer-events:\s*none/);
  });

  it('shows the art fully while revealed and dims it under verdict colour', () => {
    expect(style).toMatch(/\.tile-reveal\s+\.tile-art\s*\{[^}]*opacity:\s*1/);
    // The verdict tint must read through the art, not hide beneath it.
    expect(style).toMatch(/\.tile-correct\s+\.tile-art[^}]*opacity/);
    expect(style).toMatch(/\.tile-wrong\s+\.tile-art[^}]*opacity/);
  });

  it('never flips or moves — spatial motion is excluded entirely', () => {
    expect(style).not.toMatch(/rotate|translate|scale|perspective/);
  });
});
