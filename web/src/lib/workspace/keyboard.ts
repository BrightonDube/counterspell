// Shortcut mapping for the workspace. Enter alone never runs code, and shortcuts stay off in text fields.
export interface KeyInput {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
  /** True when focus is in an input, textarea or other form field that needs its own keys. */
  inTextField: boolean;
}

export type Shortcut = 'run' | 'submit';

/** Ctrl/Cmd+Enter runs samples. Ctrl/Cmd+Shift+Enter submits. Anything else returns null. */
export function shortcutFor(input: KeyInput): Shortcut | null {
  if (input.key !== 'Enter' || input.inTextField) return null;
  if (!input.ctrlKey && !input.metaKey) return null;
  return input.shiftKey ? 'submit' : 'run';
}
