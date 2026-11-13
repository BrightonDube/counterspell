// Tests for the browser side of sign-in (auth T29, requirements R20.5 to R20.7). The network is
// replaced by a mock, so no test calls the server, a provider or a mail service.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { AuthApiError, getMe, login, request } from './client.js';
import { ERROR_MESSAGES, messageFor } from './messages.js';
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
			vi.fn(async () =>
				new Response(JSON.stringify({ code: 'invalid_credentials', message: 'no' }), {
					status: 401,
					headers: { 'content-type': 'application/json' }
				})
			)
		);
		const failure = await login('a@example.co', 'wrong password here').catch((error) => error);
		expect(failure).toBeInstanceOf(AuthApiError);
		expect(failure.status).toBe(401);
		expect(failure.code).toBe('invalid_credentials');
	});

	it('returns null for a 204 response', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 204 })));
		await expect(request('/api/auth/logout', { method: 'POST' })).resolves.toBeNull();
	});

	it('sends the request to this site with the browser cookie and no stored credentials', async () => {
		const fetchMock = vi.fn(async () =>
			new Response(JSON.stringify({ user: { id: 'x' } }), { status: 200 })
		);
		vi.stubGlobal('fetch', fetchMock);
		await getMe();
		const call = /** @type {any} */ (fetchMock.mock.calls[0]);
		expect(call[0]).toBe('/api/me');
		expect(call[1].credentials).toBe('same-origin');
	});
});
