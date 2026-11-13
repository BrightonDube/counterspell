// Builds the Bug Autopsy view model from a parsed ghost. It shows the trusted engine's values as text.
// It does not recompute the diff, and it does not label an explanation unless the payload says where it came from.
import type { Json } from '../contract';
import { stableStringify } from './shrink-playback';
import type { GhostView } from './types';

export interface AutopsyDifference {
  path: string;
  expected: string;
  got: string;
}

export interface AutopsyViewModel {
  headline: string;
  kindLabel: string;
  input: string;
  expected: string | null;
  actual: string | null;
  actualLabel: string;
  explanation: string;
  differences: AutopsyDifference[];
  differencesTruncated: boolean;
  conceptTags: string[];
  explanationSource: {
    label: 'Authored explanation' | null;
    text: string | null;
  };
}

export type AutopsyResult =
  | { status: 'ready'; view: AutopsyViewModel }
  | { status: 'unavailable'; reason: string };

/** Shows a JSON value as compact text. Missing values are shown as an explicit word, not a guess. */
function asText(value: Json | undefined): string | null {
  return value === undefined ? null : stableStringify(value);
}

function kindCopy(kind: GhostView['kind']): {
  headline: string;
  kindLabel: string;
  actualLabel: string;
} {
  switch (kind) {
    case 'wrong':
      return {
        headline: 'Minimized counterexample',
        kindLabel: 'Wrong answer',
        actualLabel: 'Got'
      };
    case 'crash':
      return {
        headline: 'Minimized counterexample',
        kindLabel: 'Crash',
        actualLabel: 'Crash'
      };
    case 'timeout':
      // A timeout means the engine found a slow input. It did not prove a smaller failing input.
      return {
        headline: 'Slow input found',
        kindLabel: 'Slow input (timeout)',
        actualLabel: 'Timed out'
      };
  }
}

/** Builds the card model. An incomplete wrong-answer payload is unavailable, never filled in. */
export function buildAutopsy(ghost: GhostView): AutopsyResult {
  const copy = kindCopy(ghost.kind);
  const expected = asText(ghost.expected);
  const actual = asText(ghost.got);
  if (ghost.kind === 'wrong' && (expected === null || actual === null)) {
    return {
      status: 'unavailable',
      reason:
        'The expected or actual value is missing, so this card cannot compare them.'
    };
  }
  const authored = ghost.autopsy.authoredMistake;
  return {
    status: 'ready',
    view: {
      headline: copy.headline,
      kindLabel: copy.kindLabel,
      input: stableStringify(ghost.input),
      expected,
      actual,
      actualLabel: copy.actualLabel,
      explanation: ghost.explanation,
      differences: ghost.autopsy.differences.map((difference) => ({
        path: difference.path,
        expected: asText(difference.expected) ?? 'missing',
        got: asText(difference.got) ?? 'missing'
      })),
      differencesTruncated: ghost.autopsy.differencesTruncated,
      conceptTags: ghost.autopsy.conceptTags,
      explanationSource:
        authored === null
          ? { label: null, text: null }
          : { label: 'Authored explanation', text: authored.text }
    }
  };
}
