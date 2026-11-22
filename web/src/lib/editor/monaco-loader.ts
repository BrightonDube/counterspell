// Loads Monaco only when the workspace editor is needed, so the shell bundle stays small.
// The Crypt theme is defined once per page, and the editor worker is created by Monaco on demand.
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker';
import { cryptTheme } from './crypt-theme';

export const CRYPT_THEME_NAME = 'crypt';

type MonacoModule = typeof import('monaco-editor/editor/editor.api.js');

let monacoPromise: Promise<MonacoModule> | null = null;

/**
 * Imports Monaco, registers the Crypt theme and returns the Monaco module. Repeated calls share one
 * import and define the theme only once.
 */
export function loadMonaco(): Promise<MonacoModule> {
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
      import('monaco-editor/languages/definitions/python/register.js'),
      import('monaco-editor/languages/definitions/javascript/register.js'),
      import('monaco-editor/languages/definitions/java/register.js'),
      import('monaco-editor/languages/definitions/cpp/register.js'),
      import('monaco-editor/editor/contrib/find/browser/findController.js'),
      import('monaco-editor/editor/contrib/comment/browser/comment.js'),
      import('monaco-editor/editor/contrib/toggleTabFocusMode/browser/toggleTabFocusMode.js')
    ]).then(([monaco]) => {
      monaco.editor.defineTheme(CRYPT_THEME_NAME, cryptTheme);
      return monaco;
    });
    // A failed import must be retried by a later call, so the cached promise is dropped on failure.
    monacoPromise.catch(() => {
      monacoPromise = null;
    });
  }
  return monacoPromise;
}
