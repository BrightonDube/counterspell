// Local draft persistence keyed by problem slug and language. Stores only the learner's own source text.
// Never stores run events, hidden data, prompts or secrets. Storage failures keep the source in memory.
import { byteLength } from '../contract';

/** Bump this when the stored shape changes, so old entries are ignored rather than misread. */
export const DRAFT_SCHEMA_VERSION = 1;

const SAFE_SEGMENT = /^[a-z0-9][a-z0-9-]{0,127}$/;

/** The part of the Web Storage API the store uses. localStorage satisfies it. */
export interface DraftStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

export type SaveStatus = 'idle' | 'pending' | 'saved' | 'failed';

/** Builds the storage key for one problem and one language. Unsafe characters are refused. */
export function draftKey(slug: string, language: string): string {
  if (!SAFE_SEGMENT.test(slug) || !SAFE_SEGMENT.test(language)) {
    throw new Error(
      'draft key parts must be lowercase letters, digits or hyphens'
    );
  }
  return `counterspell:draft:v${DRAFT_SCHEMA_VERSION}:${slug}:${language}`;
}

function readSource(raw: string): string | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== 'object' || parsed === null) return null;
    const entry = parsed as { v?: unknown; source?: unknown };
    if (entry.v !== DRAFT_SCHEMA_VERSION || typeof entry.source !== 'string')
      return null;
    return entry.source;
  } catch {
    return null;
  }
}

/**
 * Restores a saved source. Returns null when there is none, storage cannot be read, or the source is
 * over `maxBytes`. A corrupted entry is removed so the next load starts clean.
 */
export function loadDraft(
  storage: DraftStorage,
  slug: string,
  language: string,
  maxBytes: number
): { source: string } | null {
  const key = draftKey(slug, language);
  let raw: string | null;
  try {
    raw = storage.getItem(key);
  } catch {
    return null;
  }
  if (raw === null) return null;
  const source = readSource(raw);
  if (source === null) {
    try {
      storage.removeItem(key);
    } catch {
      // Removal is best effort. The corrupted entry is ignored either way.
    }
    return null;
  }
  if (byteLength(source) > maxBytes) return null;
  return { source };
}

export interface DraftSaverOptions {
  storage: DraftStorage;
  slug: string;
  language: string;
  debounceMs: number;
}

export interface DraftSaver {
  /** Records the latest source and writes it once the pause is over. */
  schedule(source: string): void;
  /** Writes any pending source now, for example when the learner leaves the page. */
  flush(): void;
  status(): SaveStatus;
  /** The most recent source, even when storage refused the write. */
  current(): string;
  dispose(): void;
}

/** Creates a debounced saver. A failed write keeps the source in memory and sets the status to `failed`. */
export function createDraftSaver(options: DraftSaverOptions): DraftSaver {
  const key = draftKey(options.slug, options.language);
  let latest = '';
  let state: SaveStatus = 'idle';
  let timer: ReturnType<typeof setTimeout> | undefined;

  function write(): void {
    timer = undefined;
    try {
      options.storage.setItem(
        key,
        JSON.stringify({ v: DRAFT_SCHEMA_VERSION, source: latest })
      );
      state = 'saved';
    } catch {
      state = 'failed';
    }
  }

  return {
    schedule(source) {
      latest = source;
      state = 'pending';
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(write, options.debounceMs);
    },
    flush() {
      if (timer === undefined) return;
      clearTimeout(timer);
      write();
    },
    status: () => state,
    current: () => latest,
    dispose() {
      if (timer !== undefined) {
        clearTimeout(timer);
        write();
      }
    }
  };
}
