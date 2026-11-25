// Loads Monaco only when the workspace editor is needed, so the shell bundle stays small.
// The critical path is the editor API, the selected language's tokeniser, and the
// required editor features (find, comment, tab-focus): Monaco snapshots its
// contribution and action registries when the widget is constructed, so anything
// imported after loadMonaco resolves never reaches the first editor. Optional
// language packs alone prime during idle, per the workspace-performance stages.
// The Crypt theme is defined once per page, and the editor worker is created by Monaco on demand.
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker';
import { cryptTheme } from './crypt-theme';

export const CRYPT_THEME_NAME = 'crypt';

type MonacoModule = typeof import('monaco-editor/editor/editor.api.js');

const LANGUAGE_IMPORTS = {
  python: () =>
    import('monaco-editor/languages/definitions/python/register.js'),
  javascript: () =>
    import('monaco-editor/languages/definitions/javascript/register.js'),
  java: () => import('monaco-editor/languages/definitions/java/register.js'),
  cpp: () => import('monaco-editor/languages/definitions/cpp/register.js')
} as const;

// Required editor features. They load with the core so they register before
// any editor is constructed — deferred, the first editor would never get them.
const EXTRA_IMPORTS = [
  () => import('monaco-editor/editor/contrib/find/browser/findController.js'),
  () => import('monaco-editor/editor/contrib/comment/browser/comment.js'),
  () =>
    import('monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode.js')
] as const;

let monacoPromise: Promise<MonacoModule> | null = null;
const languagePromises = new Map<string, Promise<unknown>>();

/**
 * Imports one language pack, deduplicated per language and retried after a failed
 * fetch. The editor can switch models before the pack lands — the model simply
 * shows plain text until the tokeniser registers.
 */
export function loadMonacoLanguage(language: string): Promise<unknown> {
  const key = (
    language in LANGUAGE_IMPORTS ? language : 'python'
  ) as keyof typeof LANGUAGE_IMPORTS;
  let pending = languagePromises.get(key);
  if (!pending) {
    pending = LANGUAGE_IMPORTS[key]();
    languagePromises.set(key, pending);
    pending.catch(() => languagePromises.delete(key));
  }
  return pending;
}

function afterIdle(callback: () => void): void {
  const schedule = globalThis.requestIdleCallback;
  if (schedule) schedule(callback, { timeout: 5000 });
  else setTimeout(callback, 0);
}

// Optional packs prime during idle once the editor is usable, so a language
// switch still resolves quickly without blocking the shell.
function primeExtras(): void {
  for (const id of Object.keys(LANGUAGE_IMPORTS)) void loadMonacoLanguage(id);
}

/**
 * Imports Monaco plus the tokeniser for one language, registers the Crypt theme and
 * returns the Monaco module. Repeated calls share one import and define the theme
 * only once; extra languages and optional features prime during idle afterwards.
 */
export function loadMonaco(language = 'python'): Promise<MonacoModule> {
  if (!monacoPromise) {
    // Monaco only needs its editor worker for language features. Vite bundles that worker as its own file.
    const globals = globalThis as {
      MonacoEnvironment?: { getWorker: () => Worker };
    };
    globals.MonacoEnvironment ??= {
      getWorker: () => new EditorWorker()
    };
    monacoPromise = Promise.all([
      import('monaco-editor/editor/editor.api.js'),
      loadMonacoLanguage(language),
      ...EXTRA_IMPORTS.map((extra) => extra())
    ]).then(([monaco]) => {
      monaco.editor.defineTheme(CRYPT_THEME_NAME, cryptTheme);
      return monaco;
    });
    // A failed import must be retried by a later call, so the cached promise is dropped on failure.
    monacoPromise.catch(() => {
      monacoPromise = null;
    });
    void monacoPromise.then(() => afterIdle(primeExtras)).catch(() => {});
  }
  return monacoPromise;
}
