// WP03 staging contract for the Monaco loader: the selected language is the only
// pack on the critical path; the rest and the optional editor features prime
// during idle. Mocked import counters make the split observable. In its own file
// so the module registry (and the counters) start fresh.
import { describe, expect, it, vi } from 'vitest';

const h = vi.hoisted(() => ({
  imported: {} as Record<string, number>,
  idleCallbacks: [] as Array<() => void>,
  bump(key: string) {
    return () => {
      this.imported[key] = (this.imported[key] ?? 0) + 1;
      return {};
    };
  }
}));

vi.mock('monaco-editor/editor/editor.api.js', () => ({
  editor: { defineTheme() {} },
  languages: {
    register() {},
    registerTokensProviderFactory() {},
    onLanguageEncountered() {}
  }
}));
vi.mock('monaco-editor/languages/definitions/python/register.js', () =>
  h.bump('python')()
);
vi.mock('monaco-editor/languages/definitions/javascript/register.js', () =>
  h.bump('javascript')()
);
vi.mock('monaco-editor/languages/definitions/java/register.js', () =>
  h.bump('java')()
);
vi.mock('monaco-editor/languages/definitions/cpp/register.js', () =>
  h.bump('cpp')()
);
vi.mock('monaco-editor/editor/contrib/find/browser/findController.js', () =>
  h.bump('find')()
);
vi.mock('monaco-editor/editor/contrib/comment/browser/comment.js', () =>
  h.bump('comment')()
);
vi.mock(
  'monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode.js',
  () => h.bump('tabfocus')()
);

// Idle callbacks are queued in tests so priming is observable; the loader still
// requests idle scheduling in the browser.
vi.stubGlobal('requestIdleCallback', (cb: () => void) => {
  h.idleCallbacks.push(cb);
});

import { loadMonaco, loadMonacoLanguage } from './monaco-loader';

function flushIdle(): void {
  const queued = h.idleCallbacks.splice(0);
  for (const cb of queued) cb();
}

describe('Monaco readiness staging (WP03)', () => {
  it('imports only the chosen language before the editor is usable', async () => {
    await loadMonaco('javascript');
    expect(h.imported.javascript).toBe(1);
    // Extras are scheduled, not fetched on the critical path.
    expect(h.imported.python ?? 0).toBe(0);
    expect(h.imported.find ?? 0).toBe(0);
    flushIdle();
    await vi.dynamicImportSettled();
    expect(h.imported.python).toBe(1);
    expect(h.imported.java).toBe(1);
    expect(h.imported.cpp).toBe(1);
    expect(h.imported.find).toBe(1);
    expect(h.imported.comment).toBe(1);
    expect(h.imported.tabfocus).toBe(1);
  });

  // A language switch before idle priming finishes still resolves its pack, and
  // repeated switches share one in-flight import.
  it('returns one shared promise per language pack', () => {
    const first = loadMonacoLanguage('java');
    const second = loadMonacoLanguage('java');
    expect(first).toBe(second);
  });
});
