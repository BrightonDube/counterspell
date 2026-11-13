// Holds one source text per language for a problem and keeps each one saved as a local draft.
// It is plain TypeScript with no Svelte or Monaco, so it can be tested without a browser.
import {
  createDraftSaver,
  loadDraft,
  type DraftSaver,
  type DraftStorage,
  type SaveStatus
} from '../editor/draft-store';

export type SessionStatus = SaveStatus;

export interface SessionLanguage {
  id: string;
  maxSourceBytes: number;
}

export interface EditorSessionOptions {
  /** Local storage, or null when the browser has none. Null behaves like a refused write. */
  storage: DraftStorage | null;
  slug: string;
  /** Only enabled languages. The server's `enabled` flag decides which ones reach this list. */
  languages: SessionLanguage[];
  /** Starter code per language, used when no valid draft exists. */
  starters: Record<string, string>;
  debounceMs: number;
  /** Called with each save status change so the page can show it. */
  onStatus?: (language: string, status: SessionStatus) => void;
}

export interface EditorSession {
  /** The source to show for a language. Restored from a valid draft, else the starter, else empty. */
  source(language: string): string;
  /** Keeps the source in memory and schedules a save. */
  setSource(language: string, source: string): void;
  status(language: string): SessionStatus;
  /** Writes any pending drafts now, for example when the learner leaves the page. */
  dispose(): void;
}

/** Stands in for storage when there is none. Every write fails, so the status reads "failed". */
const NO_STORAGE: DraftStorage = {
  getItem: () => null,
  setItem: () => {
    throw new Error('local storage is not available');
  },
  removeItem: () => {}
};

/** Creates the session. Savers are created lazily, one per language the learner edits. */
export function createEditorSession(
  options: EditorSessionOptions
): EditorSession {
  const storage = options.storage ?? NO_STORAGE;
  const limits = new Map(
    options.languages.map((language) => [language.id, language.maxSourceBytes])
  );
  const sources = new Map<string, string>();
  const statuses = new Map<string, SessionStatus>();
  const savers = new Map<string, DraftSaver>();

  function report(language: string, status: SessionStatus) {
    statuses.set(language, status);
    options.onStatus?.(language, status);
  }

  // Reports saved or failed based on what storage actually did.
  const reporting = (language: string): DraftStorage => ({
    getItem: (key) => storage.getItem(key),
    removeItem: (key) => storage.removeItem(key),
    setItem: (key, value) => {
      try {
        storage.setItem(key, value);
        report(language, 'saved');
      } catch (error) {
        report(language, 'failed');
        throw error;
      }
    }
  });

  function saverFor(language: string): DraftSaver {
    let saver = savers.get(language);
    if (!saver) {
      saver = createDraftSaver({
        storage: reporting(language),
        slug: options.slug,
        language,
        debounceMs: options.debounceMs
      });
      savers.set(language, saver);
    }
    return saver;
  }

  return {
    source(language) {
      const known = sources.get(language);
      if (known !== undefined) return known;
      const restored = loadDraft(
        storage,
        options.slug,
        language,
        limits.get(language) ?? 0
      );
      const initial = restored?.source ?? options.starters[language] ?? '';
      sources.set(language, initial);
      return initial;
    },
    setSource(language, source) {
      sources.set(language, source);
      report(language, 'pending');
      saverFor(language).schedule(source);
    },
    status(language) {
      return statuses.get(language) ?? 'idle';
    },
    dispose() {
      for (const saver of savers.values()) saver.dispose();
    }
  };
}
