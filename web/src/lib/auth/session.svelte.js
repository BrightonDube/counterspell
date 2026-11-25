// What the browser knows about the signed-in user (auth T29). It holds only the answer of
// GET /api/me. It never holds a token, a password or a cookie value. The server decides every
// capability; the UI only reads them.
import { AuthApiError, getMe, getSessionStatus } from './client.js';

/** @typedef {'unknown' | 'signed_in' | 'signed_out'} SessionStatus */

export class SessionStore {
  /**
   * The account fields from `GET /api/me`, or `null` when nobody is signed in.
   * @type {Record<string, any> | null}
   */
  user = $state.raw(null);
  /**
   * What the account may do, from `GET /api/me`, or `null` when nobody is signed in.
   * @type {Record<string, boolean> | null}
   */
  capabilities = $state.raw(null);
  /**
   * Whether the store has asked the server yet, and what it found.
   * @type {SessionStatus}
   */
  status = $state('unknown');

  /** @type {Promise<void> | null} */
  #inflight = null;

  /**
   * Asks the server who is signed in. The status probe (`GET /api/auth/session`) always answers
   * `200`, so an expected signed-out visit never produces a `401` console error (AUD-18); only a
   * confirmed session fetches `GET /api/me`. A `401` from `/api/me` still means signed out —
   * for example a session that expired between the two calls.
   * Other failures are thrown to the caller.
   *
   * A resolved status is served from memory and concurrent callers share one request (AUD-20):
   * onboarding and account each call `load()` on mount and must not issue a second `GET /api/me`.
   * Pass `{ force: true }` to consult the server again after a state-changing action.
   *
   * @param {{ force?: boolean }} [options]
   */
  load({ force = false } = {}) {
    if (this.#inflight) return this.#inflight;
    if (this.status !== 'unknown' && !force) return Promise.resolve();
    this.#inflight = this.#fetch();
    return this.#inflight;
  }

  async #fetch() {
    try {
      const status = await getSessionStatus();
      if (!status?.signed_in) {
        this.user = null;
        this.capabilities = null;
        this.status = 'signed_out';
        return;
      }
      const data = await getMe();
      this.user = data.user;
      this.capabilities = data.capabilities;
      this.status = 'signed_in';
    } catch (error) {
      if (error instanceof AuthApiError && error.status === 401) {
        this.user = null;
        this.capabilities = null;
        this.status = 'signed_out';
        return;
      }
      throw error;
    } finally {
      this.#inflight = null;
    }
  }

  /** Forgets the user and capabilities, for example after sign-out. */
  clear() {
    this.user = null;
    this.capabilities = null;
    this.status = 'unknown';
  }
}

/** The one session store the app shares. */
export const session = new SessionStore();
