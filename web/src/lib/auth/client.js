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
	const init = { method, credentials: 'same-origin', headers: { accept: 'application/json' } };
	if (body !== undefined) {
		init.headers['content-type'] = 'application/json';
		init.body = JSON.stringify(body);
	}
	const response = await fetch(path, init);
	if (response.status === 204) return null;
	const data = await response.json().catch(() => null);
	if (!response.ok) {
		throw new AuthApiError(response.status, data?.code ?? 'internal_error', data?.message ?? '');
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
export const logoutEverywhere = () => request('/api/auth/logout-all', { method: 'POST' });
