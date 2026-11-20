// Lantern recall: a 3x3 sequence memory game. Pure logic; the page renders it.

/**
 * Adds one random tile in 0..8 to the sequence.
 * @param {number[]} sequence
 * @param {() => number} [rng]
 */
export function newRound(sequence, rng = Math.random) {
  const tile = Math.floor(rng() * 9) % 9;
  return sequence.concat(tile);
}

/**
 * Checks the press at `pressIndex` against the sequence.
 * @param {number[]} sequence
 * @param {number[]} presses
 * @param {number} pressIndex
 * @returns {'continue' | 'round_done' | 'wrong'}
 */
export function checkPress(sequence, presses, pressIndex) {
  if (presses[pressIndex] !== sequence[pressIndex]) return 'wrong';
  return pressIndex + 1 === sequence.length ? 'round_done' : 'continue';
}

/**
 * Rounds start at 3 tiles, so a finished round 3 means level 1.
 * @param {number[]} sequence
 */
export function level(sequence) {
  return sequence.length - 2;
}
