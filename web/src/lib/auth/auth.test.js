// Tests for the browser side of sign-in (auth T29, requirements R20.5 to R20.7). The network is
// replaced by a mock, so no test calls the server, a provider or a mail service.
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  AuthApiError,
  deleteAccount,
  getMe,
  providerSignInUrl,
  request,
  startProviderLink
} from './client.js';
import { SessionStore } from './session.svelte.js';
import {
  ERROR_MESSAGES,
  errorFromQuery,
  linkedMessage,
  messageFor
} from './messages.js';
import { postAuthTarget, returnToFromQuery } from './return-to.js';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('returnToFromQuery', () => {
  it('keeps an ordinary site path', () => {
    expect(returnToFromQuery('?return_to=/dashboard')).toBe('/dashboard');
  });

  it('falls back to the home page for anything that could leave the site', () => {
    const refused = [
      '?return_to=//evil.example',
      '?return_to=/%2F%2Fevil.example',
      '?return_to=/%5Cevil.example',
      '?return_to=https%3A%2F%2Fevil.example',
      '?return_to=javascript:alert(1)',
      '?return_to=/ok%0Abad'
    ];
    for (const query of refused) {
      expect(returnToFromQuery(query), query).toBe('/');
    }
  });

  it('refuses a path longer than 200 characters', () => {
    expect(returnToFromQuery(`?return_to=/${'a'.repeat(200)}`)).toBe('/');
  });

  it('falls back to the home page when there is no return_to', () => {
    expect(returnToFromQuery('')).toBe('/');
  });
});

describe('postAuthTarget', () => {
  it('sends a signed-in user to the dashboard instead of the landing page', () => {
    expect(postAuthTarget('/')).toBe('/dashboard');
  });

  it('keeps every other site path', () => {
    expect(postAuthTarget('/dashboard')).toBe('/dashboard');
    expect(postAuthTarget('/problems?x=1')).toBe('/problems?x=1');
  });
});

describe('messageFor', () => {
  it('gives a plain sentence for every code the server can send', () => {
    for (const code of Object.keys(ERROR_MESSAGES)) {
      expect(messageFor(code).length).toBeGreaterThan(0);
      expect(messageFor(code)).not.toBe(code);
    }
  });

  it('shows the generic sentence for a code it does not know', () => {
    expect(messageFor('something_new')).toBe(ERROR_MESSAGES.internal_error);
  });
});

describe('request', () => {
  it('turns the server error body into an AuthApiError carrying the code', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(
        async () =>
          new Response(
            JSON.stringify({ code: 'invalid_credentials', message: 'no' }),
            {
              status: 401,
              headers: { 'content-type': 'application/json' }
            }
          )
      )
    );
    const failure = await request('/api/me').catch((error) => error);
    expect(failure).toBeInstanceOf(AuthApiError);
    expect(failure.status).toBe(401);
    expect(failure.code).toBe('invalid_credentials');
  });

  it('returns null for a 204 response', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response(null, { status: 204 }))
    );
    await expect(
      request('/api/auth/logout', { method: 'POST' })
    ).resolves.toBeNull();
  });

  it('sends the request to this site with the browser cookie and no stored credentials', async () => {
    const fetchMock = vi.fn(
      async () =>
        new Response(JSON.stringify({ user: { id: 'x' } }), { status: 200 })
    );
    vi.stubGlobal('fetch', fetchMock);
    await getMe();
    const call = /** @type {any} */ (fetchMock.mock.calls[0]);
    expect(call[0]).toBe('/api/me');
    expect(call[1].credentials).toBe('same-origin');
  });
});

describe('endpoint calls (T29)', () => {
  /**
   * Stubs fetch with a JSON answer and returns the mock for inspection.
   * @param {any} body
   * @param {number} [status]
   */
  function answerWith(body, status = 200) {
    const fetchMock = vi.fn(
      async () =>
        new Response(body === null ? null : JSON.stringify(body), {
          status,
          headers: { 'content-type': 'application/json' }
        })
    );
    vi.stubGlobal('fetch', fetchMock);
    return fetchMock;
  }

  it('starts a provider link and returns the authorize URL the server gives', async () => {
    const fetchMock = answerWith({
      authorize_url: 'https://github.example/authorize?x=1'
    });
    const result = await startProviderLink('github');
    const [path, init] = /** @type {any} */ (fetchMock.mock.calls[0]);
    expect(path).toBe('/api/auth/link/github/start');
    expect(init.method).toBe('POST');
    expect(result.authorize_url).toBe('https://github.example/authorize?x=1');
  });

  it('builds the same-site provider sign-in URL with the return path encoded', () => {
    expect(providerSignInUrl('google', '/problems?x=1')).toBe(
      '/api/auth/oauth/google/start?return_to=%2Fproblems%3Fx%3D1'
    );
  });

  it('deletes the account with the handle as confirmation', async () => {
    const fetchMock = answerWith(null, 202);
    await deleteAccount('thandi');
    const calls = /** @type {any[]} */ (fetchMock.mock.calls);
    expect(calls[0][0]).toBe('/api/me');
    expect(calls[0][1].method).toBe('DELETE');
    expect(JSON.parse(calls[0][1].body)).toEqual({ confirm_handle: 'thandi' });
  });
});

describe('session store (T29)', () => {
  /**
   * Stubs fetch by request path, so the session probe and the `/api/me` read
   * can carry different answers.
   * @param {Record<string, { body: unknown, status?: number }>} answers
   */
  function answerPaths(answers) {
    const fetchMock = vi.fn(async (input) => {
      const raw = typeof input === 'string' ? input : input.url;
      const path = raw.startsWith('http') ? new URL(raw).pathname : raw;
      const answer = answers[path];
      return new Response(
        answer
          ? JSON.stringify(answer.body)
          : JSON.stringify({ code: 'not_found' }),
        {
          status: answer ? (answer.status ?? 200) : 404,
          headers: { 'content-type': 'application/json' }
        }
      );
    });
    vi.stubGlobal('fetch', fetchMock);
    return fetchMock;
  }

  const SIGNED_OUT = { '/api/auth/session': { body: { signed_in: false } } };
  const SIGNED_IN = {
    '/api/auth/session': { body: { signed_in: true } },
    '/api/me': {
      body: { user: { handle: 'thandi' }, capabilities: {} }
    }
  };

  it('loads the signed-in user and capabilities from GET /api/me', async () => {
    answerPaths({
      '/api/auth/session': { body: { signed_in: true } },
      '/api/me': {
        body: {
          user: { handle: 'thandi', is_demo: false, providers: ['github'] },
          capabilities: { link_provider: true }
        }
      }
    });
    const session = new SessionStore();
    await session.load();
    expect(session.user?.handle).toBe('thandi');
    expect(session.capabilities?.link_provider).toBe(true);
    expect(session.status).toBe('signed_in');
  });

  // AUD-18: the status probe always answers 200, so an expected anonymous visit
  // must never reach /api/me or log a 401.
  it('answers signed-out from the status probe without touching /api/me', async () => {
    const fetchMock = answerPaths(SIGNED_OUT);
    const session = new SessionStore();
    await session.load();
    expect(session.user).toBeNull();
    expect(session.capabilities).toBeNull();
    expect(session.status).toBe('signed_out');
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/auth/session',
      expect.anything()
    );
  });

  it('is signed out when /api/me answers 401 after a signed-in probe', async () => {
    answerPaths({
      '/api/auth/session': { body: { signed_in: true } },
      '/api/me': {
        body: { code: 'unauthenticated', message: '' },
        status: 401
      }
    });
    const session = new SessionStore();
    await session.load();
    expect(session.status).toBe('signed_out');
    expect(session.user).toBeNull();
  });

  // Breaks if a second load() re-asks the server when the status is already known —
  // onboarding and account each call load on mount, and the duplicate request is AUD-20.
  it('serves a resolved status without asking the server again', async () => {
    const fetchMock = answerPaths(SIGNED_IN);
    const session = new SessionStore();
    await session.load();
    await session.load();
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(session.status).toBe('signed_in');
  });

  // Breaks if two mounts racing load() both probe the server — concurrent callers
  // must share the whole probe sequence.
  it('shares one in-flight sequence across concurrent loads', async () => {
    const fetchMock = answerPaths(SIGNED_IN);
    const session = new SessionStore();
    await Promise.all([session.load(), session.load()]);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  // Breaks if clear() leaves a stale resolved status — after sign-out the next
  // load must consult the server again.
  it('asks the server again after the session is cleared', async () => {
    const fetchMock = answerPaths(SIGNED_IN);
    const session = new SessionStore();
    await session.load();
    session.clear();
    await session.load();
    expect(fetchMock).toHaveBeenCalledTimes(4);
  });

  // Breaks if a load started before clear() can still write its stale answer —
  // the late probe would resurrect signed_in for a user who just signed out,
  // or keep signed_out for one who just signed in (the post-login loop).
  it('does not let an in-flight load write after clear()', async () => {
    /** @type {() => void} */
    let release = () => {};
    const gate = new Promise((resolve) => {
      release = () => resolve(undefined);
    });
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input) => {
        const path = typeof input === 'string' ? input : input.url;
        if (path === '/api/auth/session') {
          await gate;
          return new Response('{"signed_in":false}', {
            status: 200,
            headers: { 'content-type': 'application/json' }
          });
        }
        return new Response('{"code":"not_found"}', { status: 404 });
      })
    );
    const session = new SessionStore();
    const pending = session.load();
    session.clear();
    release();
    await pending;
    expect(session.status).toBe('unknown');
  });

  // Breaks if a caller cannot force a refresh — dedup must never hide a stale answer.
  it('re-asks the server when forced', async () => {
    const fetchMock = answerPaths(SIGNED_IN);
    const session = new SessionStore();
    await session.load();
    await session.load({ force: true });
    expect(fetchMock).toHaveBeenCalledTimes(4);
  });

  it('clears its user when it is cleared', () => {
    const session = new SessionStore();
    session.user = { handle: 'thandi' };
    session.capabilities = { link_provider: true };
    session.clear();
    expect(session.user).toBeNull();
    expect(session.capabilities).toBeNull();
  });

  it('never writes credentials to browser storage', async () => {
    const setItem = vi.spyOn(Storage.prototype, 'setItem');
    answerPaths(SIGNED_IN);
    await new SessionStore().load();
    expect(setItem).not.toHaveBeenCalled();
  });
});

describe('errorFromQuery (T30)', () => {
  it('turns the error code in the query into its plain sentence', () => {
    expect(errorFromQuery('?error=invalid_credentials')).toBe(
      'The email or password is not correct.'
    );
  });

  it('shows nothing when there is no error code', () => {
    expect(errorFromQuery('?return_to=/problems')).toBeNull();
  });

  it('uses the generic sentence for a code the page does not know', () => {
    expect(errorFromQuery('?error=made_up')).toBe(
      ERROR_MESSAGES.internal_error
    );
  });
});

describe('linkedMessage (T32)', () => {
  it('confirms a provider that was linked', () => {
    expect(linkedMessage('?linked=github')).toBe(
      'GitHub is now linked to your account.'
    );
    expect(linkedMessage('?linked=google')).toBe(
      'Google is now linked to your account.'
    );
  });

  it('shows nothing for a provider the page does not name', () => {
    expect(linkedMessage('?linked=evil')).toBeNull();
    expect(linkedMessage('')).toBeNull();
  });
});
