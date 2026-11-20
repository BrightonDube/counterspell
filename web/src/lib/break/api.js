// Thin fetch helpers for the break-room API. Errors surface as { code }.

/**
 * @typedef {object} ApiResult
 * @property {boolean} ok
 * @property {number | string} status
 */

/**
 * @param {string} path
 * @param {RequestInit} [options]
 * @returns {Promise<ApiResult & Record<string, any>>}
 */
async function call(path, options = {}) {
  const res = await fetch(path, {
    credentials: 'same-origin',
    headers: { 'content-type': 'application/json' },
    ...options,
  });
  const body = await res.json().catch(() => ({}));
  if (!res.ok) return { ok: false, status: res.status, ...body };
  return { ok: true, status: res.status, ...body };
}

export function fetchDaily() {
  return call('/api/breakroom/puzzle/daily');
}

/** @param {Record<string, unknown>} payload */
export function saveScore(payload) {
  return call('/api/breakroom/scores', { method: 'POST', body: JSON.stringify(payload) });
}

export function fetchScores() {
  return call('/api/breakroom/scores');
}
