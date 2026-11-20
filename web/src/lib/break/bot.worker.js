// Web Worker wrapper around the minimax search. Receives { fen, seed } and
// posts back chooseMove's result. Never touches the DOM.
import { chooseMove, seededRng } from './minimax.js';

self.onmessage = (event) => {
  if (event.origin !== '' && event.origin !== self.location.origin) return;
  const { fen, seed } = event.data || {};
  try {
    const result = chooseMove(fen, { rng: seededRng(seed ?? '') });
    self.postMessage({ ok: true, ...result });
  } catch (err) {
    self.postMessage({ ok: false, error: String(err) });
  }
};
