// Tests for the lazy Monaco loader (ide-workspace T07). Monaco itself is replaced by a fake module.
import { describe, expect, it, vi } from 'vitest';

const monacoFake = vi.hoisted(() => {
  const defined: Array<[string, { base: string }]> = [];
  return {
    defined,
    editor: {
      defineTheme(name: string, theme: { base: string }) {
        defined.push([name, theme]);
      }
    }
  };
});

vi.mock('monaco-editor', () => ({ editor: monacoFake.editor }));

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
});
