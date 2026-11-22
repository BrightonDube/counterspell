// Tests for the lazy Monaco loader (ide-workspace T07). Monaco itself is replaced by a fake module.
import { readFileSync } from 'node:fs';
import { describe, expect, it, vi } from 'vitest';

const monacoFake = vi.hoisted(() => {
  const defined: Array<[string, { base: string }]> = [];
  return {
    defined,
    editor: {
      defineTheme(name: string, theme: { base: string }) {
        defined.push([name, theme]);
      }
    },
    languages: {
      register() {},
      registerTokensProviderFactory() {},
      onLanguageEncountered() {}
    }
  };
});

vi.mock('monaco-editor/editor/editor.api.js', () => ({
  editor: monacoFake.editor,
  languages: monacoFake.languages
}));

import { loadMonaco } from './monaco-loader';

describe('loadMonaco', () => {
  // Breaks if the Crypt theme is defined again on every editor mount, or never defined at all.
  it('defines the crypt theme exactly once across repeated loads', async () => {
    const first = await loadMonaco();
    const second = await loadMonaco();
    expect(first).toBe(second);
    expect(monacoFake.defined).toHaveLength(1);
    expect(monacoFake.defined[0][0]).toBe('crypt');
    expect(monacoFake.defined[0][1].base).toBe('vs-dark');
  });

  // Breaks if a future import pulls Monaco's every-language distribution into the first editor chunk.
  it('loads the standalone editor API instead of the full Monaco distribution', () => {
    const source = readFileSync('src/lib/editor/monaco-loader.ts', 'utf8');
    expect(source).toContain("import('monaco-editor/editor/editor.api.js')");
    for (const language of ['python', 'javascript', 'java', 'cpp']) {
      expect(source).toContain(
        `monaco-editor/languages/definitions/${language}/register.js`
      );
    }
    expect(source).not.toContain("import('monaco-editor')");
  });
});
