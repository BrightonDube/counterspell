// What the browser knows about the signed-in user (auth T29). It holds only the answer of
// GET /api/me. It never holds a token, a password or a cookie value. The server decides every
// capability; the UI only reads them.
import { AuthApiError, getMe } from './client.js';

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

  /**
   * Asks the server who is signed in. A `401` means signed out, which is a normal answer, not an error.
   * Other failures are thrown to the caller.
   */
  async load() {
    try {
      const data = await getMe();
      this.user = data.user;
      this.capabilities = data.capabilities;
      this.status = 'signed_in';
    } catch (error) {
      if (error instanceof AuthApiError && error.status === 401) {
        this.clear();
        this.status = 'signed_out';
        return;
      }
      throw error;
    }
  }

  /** Forgets the user and capabilities, for example after sign-out. */
  clear() {
    this.user = null;
    this.capabilities = null;
  }
}

/** The one session store the app shares. */
export const session = new SessionStore();
