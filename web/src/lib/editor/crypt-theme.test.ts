// Tests for the Monaco "Crypt" theme (ide-workspace T07). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { cryptTheme } from './crypt-theme';

/** Palette values from docs/palette.md, written out by hand so the theme cannot invent new colours. */
const PALETTE = [
  '080d0e',
  '0f1719',
  '162226',
  '23353a',
  '33494f',
  'e6e1d3',
  '9aa9a6',
  '7d8d8b',
  '8a9e8f',
  '9db3a2',
  '5d7265',
  '26332c',
  'd9a25b',
  'e8b978',
  '8fb3c0',
  '7cc458',
  '5fae3e',
  '1f3a1c',
  'e0675a',
  'c4453a',
  '3a1714',
  // Selection and scrollbar tints the palette document lists for the Monaco theme.
  '1b2a2e'
];

/** Every colour in the theme, with any alpha suffix removed and the leading # dropped. */
function allThemeColours(): string[] {
  const fromRules = cryptTheme.rules
    .map((rule) => rule.foreground)
    .filter((value): value is string => typeof value === 'string');
  const fromColors = Object.values(cryptTheme.colors ?? {}).filter(
    (value): value is string => typeof value === 'string'
  );
  return [...fromRules, ...fromColors].map((value) =>
    value.replace('#', '').slice(0, 6).toLowerCase()
  );
}

describe('cryptTheme', () => {
  // Breaks if the theme is built on a light base, which would wash out the dark Steel Night UI.
  it('starts from the dark base and inherits the Monaco defaults', () => {
    expect(cryptTheme.base).toBe('vs-dark');
    expect(cryptTheme.inherit).toBe(true);
  });

  // Breaks if a keyword loses candle amber, the second accent named in the palette.
  it('colours keywords with candle amber', () => {
    const keyword = cryptTheme.rules.find((rule) => rule.token === 'keyword');
    expect(keyword?.foreground).toBe('d9a25b');
  });

  // Breaks if the editor background drifts from the surface token the workspace panels use.
  it('uses the surface colour for the editor background', () => {
    expect(cryptTheme.colors?.['editor.background']).toBe('#0f1719');
  });

  // Breaks if a theme colour is not a documented palette value, so the editor drifts from the design.
  it('uses only documented palette colours', () => {
    const unknown = allThemeColours().filter(
      (colour) => !PALETTE.includes(colour)
    );
    expect(unknown).toEqual([]);
  });

  // Breaks if bright success green leaks into syntax colours. Green is reserved for pass states.
  it('never uses bright success green in the editor', () => {
    const colours = allThemeColours();
    expect(colours).not.toContain('7cc458');
    expect(colours).not.toContain('5fae3e');
  });

  // Breaks if invalid tokens stop using the danger red, so an error in code looks like ordinary text.
  it('marks invalid tokens with the danger colour', () => {
    const invalid = cryptTheme.rules.find((rule) => rule.token === 'invalid');
    expect(invalid?.foreground).toBe('e0675a');
  });
});
