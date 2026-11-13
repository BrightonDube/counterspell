// The measured-only Familiar state. It stores only what the server measured and sent. Code edits, typing,
// and AI speech never reach this module, so they cannot change the mood.
import type {
  ComplexityFact,
  Confidence,
  MeasuredFamiliar,
  Mood
} from './types';

export interface FamiliarState {
  mood: Mood | null;
  complexity: ComplexityFact | null;
  terminal: { verdict: string; hasBench: boolean } | null;
}

export interface FamiliarPrefs {
  muted: boolean;
  hidden: boolean;
}

export interface FamiliarView {
  /** False when the learner hid the figure. The text equivalent stays available. */
  visible: boolean;
  /** False when muted or when reduced motion is on. Only motion is removed, never information. */
  animate: boolean;
  mood: Mood | null;
  moodLabel: string;
  badge: string;
  liveText: string;
}

const MOOD_LABEL: Record<Mood, string> = {
  1: 'Furious',
  2: 'Angry',
  3: 'Uneasy',
  4: 'Pleased',
  5: 'Matches reference complexity'
};

const MOOD_WORD: Record<Mood, string> = {
  1: 'furious',
  2: 'angry',
  3: 'uneasy',
  4: 'pleased',
  5: 'laughing'
};

const BADGE: Record<Mood, string> = {
  1: 'Did not pass',
  2: 'Slower class than reference',
  3: 'Slightly slower class',
  4: 'Correct, slower constant',
  5: 'Matches reference!'
};

const CONFIDENCE_TEXT: Record<Confidence, string> = {
  low: 'low confidence, a rough reading',
  med: 'medium confidence',
  high: 'high confidence'
};

export function initialFamiliarState(): FamiliarState {
  return { mood: null, complexity: null, terminal: null };
}

/** Records a measured mood sent by the server. The layer check happens in the contract parser. */
export function applyMeasuredFamiliar(
  state: FamiliarState,
  familiar: MeasuredFamiliar
): FamiliarState {
  return { ...state, mood: familiar.mood };
}

/** Records the measured complexity fact. On its own it never sets the mood. */
export function applyComplexity(
  state: FamiliarState,
  fact: ComplexityFact
): FamiliarState {
  return { ...state, complexity: fact };
}

/** Records the terminal verdict and whether the problem has a timing check. */
export function applyTerminal(
  state: FamiliarState,
  terminal: { verdict: string; hasBench: boolean }
): FamiliarState {
  return { ...state, terminal };
}

function liveTextFor(state: FamiliarState): string {
  if (state.mood === null) {
    if (state.terminal?.verdict === 'accepted' && !state.terminal.hasBench) {
      return 'Correctness only. This problem has no timing check.';
    }
    if (state.terminal !== null && state.terminal.verdict !== 'accepted') {
      return `Did not pass: ${state.terminal.verdict}.`;
    }
    return 'Familiar: waiting for a measured result.';
  }
  const head = `Familiar: ${MOOD_WORD[state.mood]}.`;
  const fact = state.complexity;
  if (fact === null) return head;
  return `${head} Looks ${fact.measuredClass} (${CONFIDENCE_TEXT[fact.confidence]}). Reference ${fact.reference}.`;
}

/** Builds what the component shows from the measured state and the learner's preferences. */
export function familiarView(
  state: FamiliarState,
  prefs: FamiliarPrefs,
  reducedMotion: boolean
): FamiliarView {
  return {
    visible: !prefs.hidden,
    animate: !prefs.muted && !reducedMotion,
    mood: state.mood,
    moodLabel:
      state.mood === null ? 'No measured mood yet' : MOOD_LABEL[state.mood],
    badge: state.mood === null ? '' : BADGE[state.mood],
    liveText: liveTextFor(state)
  };
}
