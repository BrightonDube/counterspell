// State for the authored hint ladder. Rungs are revealed strictly in order, one request at a time.
// A response for any other rung is ignored, so a late answer can never show a hint out of order.
export interface RevealedHint {
  rung: number;
  text: string;
}

export interface LadderState {
  total: number;
  revealed: RevealedHint[];
  pending: number | null;
  error: string | null;
  capReached: boolean;
}

export function initialLadder(total: number): LadderState {
  return { total, revealed: [], pending: null, error: null, capReached: false };
}

/** The next rung to ask for, or null when one is pending, the ladder is done or the cap was reached. */
export function requestNext(state: LadderState): {
  state: LadderState;
  rung: number | null;
} {
  const done = state.revealed.length >= state.total;
  if (state.pending !== null || done || state.capReached) {
    return { state, rung: null };
  }
  const rung = state.revealed.length + 1;
  return { state: { ...state, pending: rung, error: null }, rung };
}

/** Reveals the pending rung. Any other rung is ignored. */
export function receiveRung(
  state: LadderState,
  rung: number,
  text: string
): LadderState {
  if (state.pending !== rung) return state;
  return {
    ...state,
    pending: null,
    revealed: [...state.revealed, { rung, text }]
  };
}

/** Records a failed request for the pending rung. Revealed hints are kept and no text is invented. */
export function failRung(
  state: LadderState,
  rung: number,
  message: string
): LadderState {
  if (state.pending !== rung) return state;
  return { ...state, pending: null, error: message };
}

/** Records that the daily AI cap was reached, so the ladder stops asking. */
export function markCapReached(state: LadderState): LadderState {
  return { ...state, pending: null, capReached: true };
}
