// Tests for the workspace shortcut mapping (ide-workspace T10). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { shortcutFor, type KeyInput } from './keyboard';

function key(overrides: Partial<KeyInput> = {}): KeyInput {
  return {
    key: 'Enter',
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    inTextField: false,
    ...overrides
  };
}

describe('shortcutFor', () => {
  // Breaks if the documented Run shortcut stops working with Ctrl on Windows and Linux.
  it('maps Ctrl+Enter to run', () => {
    expect(shortcutFor(key({ ctrlKey: true }))).toBe('run');
  });

  // Breaks if the Mac Command key does not run the code.
  it('maps Cmd+Enter to run', () => {
    expect(shortcutFor(key({ metaKey: true }))).toBe('run');
  });

  // Breaks if Submit and Run share a shortcut, so the learner submits when only a sample run was meant.
  it('maps Ctrl+Shift+Enter to submit', () => {
    expect(shortcutFor(key({ ctrlKey: true, shiftKey: true }))).toBe('submit');
  });

  // Breaks if a plain Enter runs the code, which would fire every time the learner types a new line.
  it('ignores a plain Enter', () => {
    expect(shortcutFor(key())).toBeNull();
  });

  // Breaks if a shortcut fires while the learner types in a form field, such as the email box.
  it('ignores shortcuts inside a text field', () => {
    expect(shortcutFor(key({ ctrlKey: true, inTextField: true }))).toBeNull();
  });
});
