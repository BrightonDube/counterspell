// Plain-words labels for run states and suggestion reasons. Status is always text, so colour is never the only
// signal. An active run shows its lifecycle status and no verdict.
import type { CardState, ReviewKind, SuggestionReason } from './types';

const VERDICT_LABELS: Record<string, string> = {
  accepted: 'Accepted',
  wrong_answer: 'Wrong answer',
  runtime_error: 'Runtime error',
  time_limit: 'Time limit',
  memory_limit: 'Memory limit',
  output_limit: 'Output limit',
  compile_error: 'Compile error'
};

const ACTIVE_LABELS: Record<string, string> = {
  queued: 'Queued',
  claimed: 'Running',
  compiling: 'Running',
  running: 'Running',
  ghost_hunting: 'Running'
};

/**
 * The label for a run. A completed run shows its verdict. An unfinished run shows its status. An internal error is
 * named as the platform's fault, not as the learner's answer.
 */
export function runStatusLabel(status: string, verdict: string | null): string {
  if (status === 'completed') {
    return (verdict !== null && VERDICT_LABELS[verdict]) || 'Finished';
  }
  if (status === 'internal_error') return 'Internal error (not a wrong answer)';
  if (status === 'expired') return 'Expired before it ran';
  if (status === 'failed') return 'Failed to run';
  return ACTIVE_LABELS[status] ?? 'In progress';
}

const REASON_LABELS: Record<SuggestionReason, string> = {
  practice_weak_concept: 'Practise a concept you missed',
  continue_track: 'Continue your track'
};

/** The words that say why a problem was suggested. */
export function reasonLabel(reason: SuggestionReason): string {
  return REASON_LABELS[reason];
}

const REVIEW_KIND_LABELS: Record<ReviewKind, string> = {
  sibling: 'Practises the same concept',
  normal: 'General challenge'
};

/** How a review challenge was selected, so a review reads differently from a suggestion. */
export function reviewKindLabel(kind: ReviewKind): string {
  return REVIEW_KIND_LABELS[kind];
}

const CARD_STATE_LABELS: Record<CardState, string> = {
  new: 'New',
  learning: 'Learning',
  review: 'Review',
  relearning: 'Relearning'
};

/** The words for a review card's scheduler state. */
export function cardStateLabel(state: CardState): string {
  return CARD_STATE_LABELS[state];
}
