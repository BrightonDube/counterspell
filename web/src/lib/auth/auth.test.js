// Tests for the browser side of sign-in (auth T29, requirements R20.5 to R20.7). The network is
// replaced by a mock, so no test calls the server, a provider or a mail service.
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  AuthApiError,
  confirmPasswordReset,
  deleteAccount,
  getMe,
  login,
  providerSignInUrl,
  register,
  request,
  requestEmailLink,
  requestMagicLink,
  requestPasswordReset,
  startProviderLink,
  verifyMagicLink
} from './client.js';
import { SessionStore } from './session.svelte.js';
import {
  ERROR_MESSAGES,
  errorFromQuery,
  linkedMessage,
  messageFor
} from './messages.js';
import { returnToFromQuery } from './return-to.js';

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
    const failure = await login('a@example.co', 'wrong password here').catch(
      (error) => error
    );
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

  it('posts a magic-link request with the email and the sanitised return path', async () => {
    const fetchMock = answerWith(null, 202);
    await requestMagicLink('thandi@example.co', '/problems');
    const [path, init] = /** @type {any} */ (fetchMock.mock.calls[0]);
    expect(path).toBe('/api/auth/email/magic');
    expect(init.method).toBe('POST');
    expect(JSON.parse(init.body)).toEqual({
      email: 'thandi@example.co',
      return_to: '/problems'
    });
  });

  it('sends the token from the link to the verify endpoint and nowhere else', async () => {
    const fetchMock = answerWith({
      user: { handle: 'thandi' },
      return_to: '/'
    });
    await verifyMagicLink('abc123');
    const [path, init] = /** @type {any} */ (fetchMock.mock.calls[0]);
    expect(path).toBe('/api/auth/email/verify');
    expect(JSON.parse(init.body)).toEqual({ token: 'abc123' });
  });

  it('posts password login, register and reset calls with their bodies', async () => {
    const fetchMock = answerWith(null, 202);
    await register('a@example.co', 'a long passphrase', '/');
    await requestPasswordReset('a@example.co');
    await confirmPasswordReset('tok', 'another passphrase');
    const calls = /** @type {any[]} */ (fetchMock.mock.calls);
    expect(calls.map((call) => call[0])).toEqual([
      '/api/auth/register',
      '/api/auth/password/reset',
      '/api/auth/password/reset/confirm'
    ]);
    expect(JSON.parse(calls[0][1].body)).toEqual({
      email: 'a@example.co',
      password: 'a long passphrase',
      return_to: '/'
    });
    expect(JSON.parse(calls[1][1].body)).toEqual({ email: 'a@example.co' });
    expect(JSON.parse(calls[2][1].body)).toEqual({
      token: 'tok',
      password: 'another passphrase'
    });
  });

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

  it('requests an email link for linking and deletes the account with the handle as confirmation', async () => {
    const fetchMock = answerWith(null, 202);
    await requestEmailLink('new@example.co');
    await deleteAccount('thandi');
    const calls = /** @type {any[]} */ (fetchMock.mock.calls);
    expect(calls[0][0]).toBe('/api/auth/link/email');
    expect(JSON.parse(calls[0][1].body)).toEqual({ email: 'new@example.co' });
    expect(calls[1][0]).toBe('/api/me');
    expect(calls[1][1].method).toBe('DELETE');
    expect(JSON.parse(calls[1][1].body)).toEqual({ confirm_handle: 'thandi' });
  });
});

describe('session store (T29)', () => {
  it('loads the signed-in user and capabilities from GET /api/me', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(
        async () =>
          new Response(
            JSON.stringify({
              user: { handle: 'thandi', is_demo: false, providers: ['github'] },
              capabilities: { link_provider: true }
            }),
            { status: 200 }
          )
      )
    );
    const session = new SessionStore();
    await session.load();
    expect(session.user?.handle).toBe('thandi');
    expect(session.capabilities?.link_provider).toBe(true);
    expect(session.status).toBe('signed_in');
  });

  it('is signed out with no user after a 401', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(
        async () =>
          new Response(
            JSON.stringify({ code: 'unauthenticated', message: '' }),
            {
              status: 401
            }
          )
      )
    );
    const session = new SessionStore();
    await session.load();
    expect(session.user).toBeNull();
    expect(session.capabilities).toBeNull();
    expect(session.status).toBe('signed_out');
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
    vi.stubGlobal(
      'fetch',
      vi.fn(
        async () =>
          new Response(JSON.stringify({ user: {}, capabilities: {} }), {
            status: 200
          })
      )
    );
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
