// Tests for per-problem, per-language draft persistence (ide-workspace T08). Each test names the break it catches.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  createDraftSaver,
  draftKey,
  loadDraft,
  type DraftStorage
} from './draft-store';
import { checkSourceSize, copySource } from './source-validation';

/** An in-memory storage with an optional failure switch, standing in for localStorage in tests. */
function memoryStorage(
  options: { failWrites?: boolean; failReads?: boolean } = {}
): DraftStorage & {
  entries: Map<string, string>;
} {
  const entries = new Map<string, string>();
  return {
    entries,
    getItem(key) {
      if (options.failReads) throw new Error('storage disabled');
      return entries.get(key) ?? null;
    },
    setItem(key, value) {
      if (options.failWrites) throw new Error('quota exceeded');
      entries.set(key, value);
    },
    removeItem(key) {
      entries.delete(key);
    }
  };
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('draftKey', () => {
  // Breaks if two languages share one key, so a Python draft would overwrite a JavaScript draft.
  it('keeps each language in its own key', () => {
    expect(draftKey('two-sum', 'python')).not.toBe(
      draftKey('two-sum', 'javascript')
    );
  });

  // Breaks if a slug with path or separator characters can collide with another problem's key.
  it('refuses a slug or language with characters outside the safe set', () => {
    expect(() => draftKey('two-sum/../x', 'python')).toThrow();
    expect(() => draftKey('two-sum', 'py:thon')).toThrow();
  });
});

describe('saving and restoring drafts', () => {
  // Breaks if switching languages loses the first language's source or loads it into the other.
  it('restores the draft for the language that was saved, not another language', () => {
    const storage = memoryStorage();
    const saver = createDraftSaver({
      storage,
      slug: 'two-sum',
      language: 'python',
      debounceMs: 0
    });
    saver.schedule('def two_sum(nums): return []');
    vi.runAllTimers();
    expect(loadDraft(storage, 'two-sum', 'python', 65_536)).toEqual({
      source: 'def two_sum(nums): return []'
    });
    expect(loadDraft(storage, 'two-sum', 'javascript', 65_536)).toBeNull();
  });

  // Breaks if rapid typing writes to storage on every keystroke instead of once after the pause.
  it('writes once after the debounce pause with the last value', () => {
    const storage = memoryStorage();
    const setItem = vi.spyOn(storage, 'setItem');
    const saver = createDraftSaver({
      storage,
      slug: 'two-sum',
      language: 'python',
      debounceMs: 500
    });
    saver.schedule('a');
    vi.advanceTimersByTime(200);
    saver.schedule('ab');
    vi.advanceTimersByTime(200);
    saver.schedule('abc');
    vi.advanceTimersByTime(499);
    expect(setItem).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(setItem).toHaveBeenCalledTimes(1);
    expect(loadDraft(storage, 'two-sum', 'python', 65_536)?.source).toBe('abc');
  });

  // Breaks if a full storage quota discards the learner's source instead of keeping it in memory.
  it('keeps the in-memory source and reports failed when the quota is full', () => {
    const storage = memoryStorage({ failWrites: true });
    const saver = createDraftSaver({
      storage,
      slug: 'two-sum',
      language: 'python',
      debounceMs: 0
    });
    saver.schedule('keep me');
    vi.runAllTimers();
    expect(saver.status()).toBe('failed');
    expect(saver.current()).toBe('keep me');
  });

  // Breaks if an unreadable storage (for example private mode) throws into the editor and blocks typing.
  it('treats unreadable storage as no draft', () => {
    const storage = memoryStorage({ failReads: true });
    expect(loadDraft(storage, 'two-sum', 'python', 65_536)).toBeNull();
  });

  // Breaks if a corrupted entry is kept forever, so the learner never gets a working starter back.
  it('removes a corrupted entry and returns no draft', () => {
    const storage = memoryStorage();
    storage.entries.set(draftKey('two-sum', 'python'), '{not json');
    expect(loadDraft(storage, 'two-sum', 'python', 65_536)).toBeNull();
    expect(storage.entries.has(draftKey('two-sum', 'python'))).toBe(false);
  });

  // Breaks if a stored source larger than the byte limit is restored and sent past the editor cap.
  it('ignores a stored source over the byte limit', () => {
    const storage = memoryStorage();
    const saver = createDraftSaver({
      storage,
      slug: 'two-sum',
      language: 'python',
      debounceMs: 0
    });
    saver.schedule('x'.repeat(100));
    vi.runAllTimers();
    expect(loadDraft(storage, 'two-sum', 'python', 50)).toBeNull();
  });
});

describe('source byte limits', () => {
  // Breaks if the count uses characters, so a multibyte source passes the server's byte limit in the browser.
  it('counts UTF-8 bytes, not characters', () => {
    // Each emoji is 4 bytes and 2 UTF-16 code units. Three emoji are 12 bytes.
    expect(checkSourceSize('😀😀😀', 11)).toEqual({
      ok: false,
      bytes: 12,
      limit: 11,
      message:
        'This solution is 12 bytes. The limit for this language is 11 bytes.'
    });
  });

  // Breaks if an accented character is counted as one byte, which would undercount the limit.
  it('counts an accented letter as two bytes', () => {
    expect(checkSourceSize('é', 1)).toMatchObject({ ok: false, bytes: 2 });
    expect(checkSourceSize('é', 2)).toEqual({ ok: true, bytes: 2 });
  });
});

describe('copy fallback', () => {
  // Breaks if a clipboard refusal is reported as a successful copy, so the learner loses the source silently.
  it('reports a failed copy when the clipboard refuses the write', async () => {
    const clipboard = { writeText: () => Promise.reject(new Error('denied')) };
    expect(await copySource('print(1)', clipboard)).toBe(false);
  });

  // Breaks if a working clipboard is not used, so the copy fallback never delivers the source.
  it('writes the exact source to the clipboard', async () => {
    const written: string[] = [];
    const clipboard = {
      writeText: async (text: string) => {
        written.push(text);
      }
    };
    expect(await copySource('print(1)', clipboard)).toBe(true);
    expect(written).toEqual(['print(1)']);
  });
});
