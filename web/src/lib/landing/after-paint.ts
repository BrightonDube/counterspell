// Runs work after the first paint, so optional motion and video never delay the page.

/** Longest wait before the callback runs when the browser never reports an idle period. */
const MAX_DELAY_MS = 1500;

/**
 * Calls `callback` once the browser is idle after first paint.
 * Where idle callbacks are missing (older browsers), it falls back to a timer of MAX_DELAY_MS.
 */
export function afterPaint(callback: () => void): void {
  if (typeof requestIdleCallback === 'function') {
    requestIdleCallback(() => callback(), { timeout: MAX_DELAY_MS });
  } else {
    setTimeout(callback, MAX_DELAY_MS);
  }
}
