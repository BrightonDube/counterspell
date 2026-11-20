// Depth-2 negamax with alpha-beta pruning, material-only evaluation.
// Deterministic: a node budget caps work and a seeded rng breaks ties;
// a wall-clock deadline only guards slow devices.
import { Chess } from 'chess.js';

/** @type {Record<string, number>} */
const PIECE_VALUES = { p: 100, n: 320, b: 330, r: 500, q: 900, k: 0 };
const MATE = 100000;

/** @param {Chess} game */
function evaluate(game) {
  let score = 0;
  for (const row of game.board()) {
    for (const sq of row) {
      if (!sq) continue;
      const v = PIECE_VALUES[sq.type] || 0;
      score += sq.color === 'w' ? v : -v;
    }
  }
  return score;
}

/** Captures first by victim value, then the order chess.js returns.
 * @param {Chess} game
 */
function orderedMoves(game) {
  return game
    .moves({ verbose: true })
    .map((m) => ({ m, victim: m.captured ? PIECE_VALUES[m.captured] || 0 : 0 }))
    .sort((a, b) => b.victim - a.victim)
    .map((x) => x.m);
}

/**
 * Picks the bot's move for a position.
 * @param {string} fen
 * @param {{ depth?: number, nodeBudget?: number, deadlineMs?: number, rng?: () => number }} [options]
 * @returns {{ uci: string | null, san: string | null, nodes: number, timedOut: boolean }}
 */
export function chooseMove(fen, options = {}) {
  const depth = options.depth ?? 2;
  const nodeBudget = options.nodeBudget ?? 60000;
  const deadlineMs = options.deadlineMs ?? 1000;
  const rng = options.rng ?? Math.random;
  const game = new Chess(fen);
  const deadline = Date.now() + deadlineMs;
  const state = { nodes: 0, timedOut: false };
  const us = game.turn() === 'w' ? 1 : -1;

  /** @param {number} ply @param {number} alpha @param {number} beta */
  function search(ply, alpha, beta) {
    state.nodes += 1;
    if (state.nodes >= nodeBudget || Date.now() > deadline) {
      state.timedOut = true;
      return us * evaluate(game);
    }
    if (game.isCheckmate()) return -(MATE - ply);
    if (game.isDraw() || game.isStalemate()) return 0;
    if (ply >= depth) return us * evaluate(game);
    let best = -Infinity;
    for (const move of orderedMoves(game)) {
      game.move(move);
      const score = -search(ply + 1, -beta, -alpha);
      game.undo();
      if (score > best) best = score;
      if (best > alpha) alpha = best;
      if (alpha >= beta) break;
    }
    return best;
  }

  const moves = orderedMoves(game);
  if (moves.length === 0)
    return {
      uci: null,
      san: null,
      nodes: state.nodes,
      timedOut: state.timedOut
    };
  let bestScore = -Infinity;
  /** @type {import('chess.js').Move[]} */
  let best = [];
  for (const move of moves) {
    game.move(move);
    const score = -search(1, -Infinity, -Infinity);
    game.undo();
    if (score > bestScore) {
      bestScore = score;
      best = [move];
    } else if (score === bestScore) {
      best.push(move);
    }
  }
  const pick = best[Math.floor(rng() * best.length) % best.length];
  return {
    uci: pick.from + pick.to + (pick.promotion ?? ''),
    san: pick.san,
    nodes: state.nodes,
    timedOut: state.timedOut
  };
}

/** A deterministic PRNG so the bot is repeatable in tests and per game id.
 * @param {unknown} seed
 */
export function seededRng(seed) {
  let s = 0;
  for (const ch of String(seed)) s = (s * 31 + ch.charCodeAt(0)) >>> 0;
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0;
    return s / 0x100000000;
  };
}
