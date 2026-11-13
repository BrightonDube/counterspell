// Fetch wrapper for the auth API. Same-origin only, JSON only. Nothing is stored in browser
// storage: the session lives in an HttpOnly cookie that JavaScript cannot read.

/** An error answer from the auth API, with its stable code. */
export class AuthApiError extends Error {
  /**
   * @param {number} status the HTTP status
   * @param {string} code the stable error code, for example `invalid_credentials`
   * @param {string} message the server's plain sentence (shown only as a fallback)
   */
  constructor(status, code, message) {
    super(message);
    this.name = 'AuthApiError';
    this.status = status;
    this.code = code;
  }
}

/**
 * Sends one request to this site and returns the parsed JSON, or `null` for `204`.
 * @param {string} path a path on this site
 * @param {{ method?: string, body?: unknown }} [options]
 */
export async function request(path, { method = 'GET', body } = {}) {
  /** @type {RequestInit & { headers: Record<string, string> }} */
  const init = {
    method,
    credentials: 'same-origin',
    headers: { accept: 'application/json' }
  };
  if (body !== undefined) {
    init.headers['content-type'] = 'application/json';
    init.body = JSON.stringify(body);
  }
  const response = await fetch(path, init);
  if (response.status === 204) return null;
  const data = await response.json().catch(() => null);
  if (!response.ok) {
    throw new AuthApiError(
      response.status,
      data?.code ?? 'internal_error',
      data?.message ?? ''
    );
  }
  return data;
}

/** `GET /api/auth/config`: which sign-in options are offered. */
export const getConfig = () => request('/api/auth/config');
/** `GET /api/me`: the signed-in user and capabilities. */
export const getMe = () => request('/api/me');
/** `POST /api/auth/login`: email and password sign-in. */
/**
 * @param {string} email
 * @param {string} password
 */
export const login = (email, password) =>
  request('/api/auth/login', { method: 'POST', body: { email, password } });
/** `POST /api/auth/logout`: ends this browser's session. */
export const logout = () => request('/api/auth/logout', { method: 'POST' });
/** `POST /api/auth/logout-all`: ends every session of the user. */
export const logoutEverywhere = () =>
  request('/api/auth/logout-all', { method: 'POST' });

/**
 * `POST /api/auth/email/magic`: asks for a sign-in link. The answer is the same for every address.
 * @param {string} email
 * @param {string} returnTo a site path, sanitised again by the server
 */
export const requestMagicLink = (email, returnTo) =>
  request('/api/auth/email/magic', {
    method: 'POST',
    body: { email, return_to: returnTo }
  });

/**
 * `POST /api/auth/email/verify`: uses a link token. Called only after the user presses confirm.
 * @param {string} token the single-use token from the link fragment
 */
export const verifyMagicLink = (token) =>
  request('/api/auth/email/verify', { method: 'POST', body: { token } });

/**
 * `POST /api/auth/register` (Should): starts a password registration that needs a mailbox check.
 * @param {string} email
 * @param {string} password
 * @param {string} returnTo
 */
export const register = (email, password, returnTo) =>
  request('/api/auth/register', {
    method: 'POST',
    body: { email, password, return_to: returnTo }
  });

/**
 * `POST /api/auth/password/reset` (Should): asks for a reset link. The answer is the same for every address.
 * @param {string} email
 */
export const requestPasswordReset = (email) =>
  request('/api/auth/password/reset', { method: 'POST', body: { email } });

/**
 * `POST /api/auth/password/reset/confirm` (Should): sets a new password from a reset token.
 * @param {string} token
 * @param {string} password
 */
export const confirmPasswordReset = (token, password) =>
  request('/api/auth/password/reset/confirm', {
    method: 'POST',
    body: { token, password }
  });

/**
 * The URL that starts a provider sign-in. The browser follows it with a plain link, not fetch,
 * because the server answers with a redirect to the provider.
 * @param {string} provider `github` or `google`
 * @param {string} returnTo a site path
 */
export const providerSignInUrl = (provider, returnTo) =>
  `/api/auth/oauth/${provider}/start?return_to=${encodeURIComponent(returnTo)}`;

/**
 * `POST /api/auth/link/{provider}/start`: starts linking a provider to the signed-in account.
 * @param {string} provider `github` or `google`
 * @returns {Promise<{ authorize_url: string }>} the provider page to visit
 */
export const startProviderLink = (provider) =>
  request(`/api/auth/link/${provider}/start`, { method: 'POST' });

/**
 * `POST /api/auth/link/email`: sends a verification link to add an email to the account.
 * @param {string} email
 */
export const requestEmailLink = (email) =>
  request('/api/auth/link/email', { method: 'POST', body: { email } });

/**
 * `DELETE /api/me` (Could): deletes the account. The handle must be typed again as confirmation.
 * @param {string} confirmHandle the account's own handle
 */
export const deleteAccount = (confirmHandle) =>
  request('/api/me', {
    method: 'DELETE',
    body: { confirm_handle: confirmHandle }
  });
