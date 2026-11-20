// Daily-puzzle state machine. The first solution move is the opponent's
// and is played automatically; each user move is checked in UCI form after
// chess.js confirms legality. Illegal moves never count as mistakes.
import { Chess } from 'chess.js';

/**
 * @typedef {object} PuzzleState
 * @property {Chess} game
 * @property {string[]} solution remaining solution moves in UCI
 * @property {number} index next solution move to play or verify
 * @property {number} mistakes capped at 20
 * @property {'playing' | 'solved' | 'failed'} status
 */

/**
 * Starts a puzzle: loads the FEN and plays the opponent's first solution move.
 * @param {string} fen
 * @param {string[]} solutionMoves
 * @returns {PuzzleState}
 */
export function newPuzzle(fen, solutionMoves) {
  const game = new Chess(fen);
  /** @type {PuzzleState} */
  const state = { game, solution: solutionMoves.slice(), index: 0, mistakes: 0, status: 'playing' };
  playOpponent(state);
  return state;
}

/** @param {PuzzleState} state */
function playOpponent(state) {
  if (state.index < state.solution.length) {
    state.game.move(uciToMove(state.solution[state.index]));
    state.index += 1;
  }
}

/** @param {string} uci */
function uciToMove(uci) {
  /** @type {{ from: string, to: string, promotion?: string }} */
  const move = { from: uci.slice(0, 2), to: uci.slice(2, 4) };
  if (uci.length > 4) move.promotion = uci[4];
  return move;
}

/**
 * Tries a user move (UCI like `e2e4`, or SAN like `Nf3`).
 * @param {PuzzleState} state
 * @param {string} input
 * @returns {{ state: PuzzleState, feedback: string }}
 */
export function tryMove(state, input) {
  if (state.status !== 'playing') return { state, feedback: 'The puzzle is already finished.' };
  /** @type {import('chess.js').Move} */
  let played;
  try {
    played = state.game.move(input.length <= 5 ? uciToMove(input) : input);
  } catch {
    try {
      played = state.game.move(input); // SAN path
    } catch {
      return { state, feedback: 'Illegal move.' };
    }
  }
  const expected = state.solution[state.index];
  const uci = played.from + played.to + (played.promotion ?? '');
  if (uci !== expected) {
    state.game.undo();
    state.mistakes = Math.min(20, state.mistakes + 1);
    return { state, feedback: 'Not the solution. Try again.' };
  }
  state.index += 1;
  if (state.index >= state.solution.length) {
    state.status = 'solved';
    return { state, feedback: 'Solved.' };
  }
  playOpponent(state);
  if (state.index >= state.solution.length) state.status = 'solved';
  return { state, feedback: 'Correct.' };
}
