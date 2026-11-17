import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { REQUIRED_IDS, validate } from './contract';

const HERE = dirname(fileURLToPath(import.meta.url));

/** An SVG that carries every required id, and nothing else. */
function svgWith(ids: readonly string[]): string {
  const groups = ids.map((id) => `<g id="${id}"></g>`).join('');
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">${groups}</svg>`;
}

describe('familiar asset contract (approved mood paintings, 15 Nov 2026)', () => {
  // Breaks if the contract asks for carved eye and mouth layers the approved paintings do not have.
  it('lists the ids the realistic art needs: body, glow stops, five mood images and the effect layer', () => {
    expect(REQUIRED_IDS).toEqual([
      'fam-body',
      'fam-glow-stop-inner',
      'fam-glow-stop-outer',
      'fam-mood-1',
      'fam-mood-2',
      'fam-mood-3',
      'fam-mood-4',
      'fam-mood-5',
      'fam-fx'
    ]);
  });

  // Breaks if the shipped mascot file drifts from the contract, so the mascot would fall back to the placeholder.
  it('the shipped jack-o-lantern.svg meets the contract', () => {
    const art = readFileSync(
      join(
        HERE,
        '..',
        '..',
        '..',
        '..',
        'static',
        'familiar',
        'jack-o-lantern.svg'
      ),
      'utf8'
    );
    expect(validate(art)).toEqual({ ok: true, missing: [] });
  });

  it('accepts an SVG that has every required id', () => {
    expect(validate(svgWith(REQUIRED_IDS))).toEqual({ ok: true, missing: [] });
  });

  it('names the first missing id, so the failure says what to add', () => {
    const withoutMood3 = REQUIRED_IDS.filter((id) => id !== 'fam-mood-3');
    expect(validate(svgWith(withoutMood3))).toEqual({
      ok: false,
      missing: ['fam-mood-3']
    });
  });

  it('names every missing id, not only the first', () => {
    const withoutTwo = REQUIRED_IDS.filter(
      (id) => id !== 'fam-mood-3' && id !== 'fam-mood-5'
    );
    expect(validate(svgWith(withoutTwo)).missing).toEqual([
      'fam-mood-3',
      'fam-mood-5'
    ]);
  });

  it('does not accept an id that only appears as a prefix of another id', () => {
    const prefixOnly = REQUIRED_IDS.map((id) =>
      id === 'fam-fx' ? 'fam-fx-extra' : id
    );
    expect(validate(svgWith(prefixOnly))).toEqual({
      ok: false,
      missing: ['fam-fx']
    });
  });
});
