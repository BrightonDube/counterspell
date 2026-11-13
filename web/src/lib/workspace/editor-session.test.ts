// Tests for the editor session that keeps one draft per problem and language (ide-workspace T09).
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { draftKey } from '../editor/draft-store';
import { createEditorSession, type SessionStatus } from './editor-session';

/** An in-memory stand-in for localStorage, with a switch that makes every write fail. */
function memoryStorage(initial: Record<string, string> = {}) {
  const items = new Map(Object.entries(initial));
  const refuse = { writes: false };
  return {
    items,
    refuse,
    getItem: (key: string) => items.get(key) ?? null,
    setItem: (key: string, value: string) => {
      if (refuse.writes) throw new Error('quota exceeded');
      items.set(key, value);
    },
    removeItem: (key: string) => {
      items.delete(key);
    }
  };
}

const LANGUAGES = [
  { id: 'python', maxSourceBytes: 65536 },
  { id: 'javascript', maxSourceBytes: 65536 }
];
const STARTERS = {
  python: 'def two_sum(nums, target):\n    pass\n',
  javascript: 'function twoSum(nums, target) {}\n'
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('createEditorSession', () => {
  // Breaks if a saved draft is ignored, so a reload throws away the learner's work.
  it('restores the saved draft for the language it was written in', () => {
    const storage = memoryStorage({
      [draftKey('two-sum', 'python')]: JSON.stringify({
        v: 1,
        source: 'saved = 1'
      })
    });
    const session = createEditorSession({
      storage,
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400
    });
    expect(session.source('python')).toBe('saved = 1');
    expect(session.source('javascript')).toBe(STARTERS.javascript);
  });

  // Breaks if a new learner sees an empty editor instead of the starter signature.
  it('falls back to the starter when there is no draft', () => {
    const session = createEditorSession({
      storage: memoryStorage(),
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400
    });
    expect(session.source('python')).toBe(STARTERS.python);
  });

  // Breaks if editing Python overwrites the JavaScript draft, which the spec says must stay separate.
  it('keeps the drafts for each language separate', () => {
    const session = createEditorSession({
      storage: memoryStorage(),
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400
    });
    session.setSource('python', 'py = 1');
    session.setSource('javascript', 'js = 2');
    expect(session.source('python')).toBe('py = 1');
    expect(session.source('javascript')).toBe('js = 2');
  });

  // Breaks if every keystroke is written, or the latest text is lost after the pause.
  it('writes only the latest source once the pause is over', () => {
    const storage = memoryStorage();
    const session = createEditorSession({
      storage,
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400
    });
    session.setSource('python', 'a = 1');
    session.setSource('python', 'a = 12');
    expect(storage.items.has(draftKey('two-sum', 'python'))).toBe(false);

    vi.advanceTimersByTime(400);
    expect(
      JSON.parse(storage.items.get(draftKey('two-sum', 'python')) ?? '{}')
    ).toEqual({
      v: 1,
      source: 'a = 12'
    });
    expect(session.status('python')).toBe('saved');
  });

  // Breaks if a refused write drops the source, or the status stays "saved" when nothing was stored.
  it('keeps the source in memory and reports failure when storage refuses the write', () => {
    const storage = memoryStorage();
    storage.refuse.writes = true;
    const statuses: SessionStatus[] = [];
    const session = createEditorSession({
      storage,
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400,
      onStatus: (_language, status) => statuses.push(status)
    });
    session.setSource('python', 'kept = 1');
    vi.advanceTimersByTime(400);

    expect(session.source('python')).toBe('kept = 1');
    expect(session.status('python')).toBe('failed');
    expect(statuses).toEqual(['pending', 'failed']);
  });

  // Breaks if an oversized draft is restored, so the learner would see code the server then refuses.
  it('ignores a stored draft larger than the language limit', () => {
    const storage = memoryStorage({
      [draftKey('two-sum', 'python')]: JSON.stringify({
        v: 1,
        source: 'x'.repeat(70000)
      })
    });
    const session = createEditorSession({
      storage,
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400
    });
    expect(session.source('python')).toBe(STARTERS.python);
  });

  // Breaks if a corrupted entry crashes the editor instead of starting from the starter.
  it('recovers from a corrupted draft by starting from the starter', () => {
    const storage = memoryStorage({
      [draftKey('two-sum', 'python')]: '{not json'
    });
    const session = createEditorSession({
      storage,
      slug: 'two-sum',
      languages: LANGUAGES,
      starters: STARTERS,
      debounceMs: 400
    });
    expect(session.source('python')).toBe(STARTERS.python);
  });
});
