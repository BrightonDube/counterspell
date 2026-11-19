// The hint ladder state for the IDE (hints spec H15). This is pure: it holds no DOM and no network.
// It mirrors the server gate: rung N can be asked for only when rung N-1 is revealed in the ladder,
// and AI hints start at rung 2.
import type { HintQuota, HintReply } from './client';

/** An AI hint the panel shows, with its label and provider. */
export interface AiHintEntry {
  body: string;
  label: string;
  provider: string | null;
  model: string | null;
}

/** What the panel knows about one submission's hints. */
export interface HintPanelState {
  /** Authored rungs the learner has revealed, keyed by rung number. */
  ladder: Record<number, string>;
  /** AI hints the learner has asked for, keyed by rung number. */
  ai: Record<number, AiHintEntry>;
  /** The next authored rung, or `null` when all four are revealed. */
  nextRung: number | null;
  /** The latest quota the server reported. */
  quota: HintQuota | null;
  /** The rung with a request in flight, or `null`. */
  pending: number | null;
  /** A message to show after a failed request, or `null`. */
  error: string | null;
}

/** Events that change the panel state. */
export type HintAction =
  | { type: 'start'; rung: number; ai: boolean }
  | { type: 'reply'; reply: HintReply }
  | { type: 'failed'; message: string };

/** Shown next to the AI button. The text says where the learner's code goes before it is sent. */
export const AI_NOTICE =
  'Asking for an AI hint sends your code and the ghost input to an AI provider.';

const LAST_RUNG = 4;

/** The state before anything is revealed. */
export function initialHintState(): HintPanelState {
  return {
    ladder: {},
    ai: {},
    nextRung: 1,
    quota: null,
    pending: null,
    error: null
  };
}

/**
 * Whether a request for `rung` may be sent now. A rung that already has text is not requested again.
 * An AI hint needs rung 2 or later and the rung before it in the ladder.
 */
export function canRequest(
  state: HintPanelState,
  rung: number,
  ai: boolean
): boolean {
  if (!Number.isInteger(rung) || rung < 1 || rung > LAST_RUNG) return false;
  if (state.pending !== null) return false;
  if (state.ladder[rung] !== undefined || state.ai[rung] !== undefined)
    return false;
  if (ai && rung < 2) return false;
  return rung === 1 || state.ladder[rung - 1] !== undefined;
}

/** Applies one event and returns the next state. A start that `canRequest` refuses changes nothing. */
export function hintReducer(
  state: HintPanelState,
  action: HintAction
): HintPanelState {
  switch (action.type) {
    case 'start':
      if (!canRequest(state, action.rung, action.ai)) return state;
      return { ...state, pending: action.rung, error: null };
    case 'reply':
      return applyReply(state, action.reply);
    case 'failed':
      return { ...state, pending: null, error: action.message };
  }
}

function applyReply(state: HintPanelState, reply: HintReply): HintPanelState {
  const quota = reply.quota ?? state.quota;
  if (reply.source === 'ai') {
    return {
      ...state,
      ai: {
        ...state.ai,
        [reply.rung]: {
          body: reply.body,
          label: reply.label ?? 'AI hint',
          provider: reply.provider,
          model: reply.model
        }
      },
      quota,
      pending: null
    };
  }
  const ladder = { ...state.ladder, [reply.rung]: reply.body };
  return {
    ...state,
    ladder,
    nextRung: reply.nextRung ?? firstUnrevealed(ladder),
    quota,
    pending: null
  };
}

function firstUnrevealed(ladder: Record<number, string>): number | null {
  for (let rung = 1; rung <= LAST_RUNG; rung++) {
    if (ladder[rung] === undefined) return rung;
  }
  return null;
}
