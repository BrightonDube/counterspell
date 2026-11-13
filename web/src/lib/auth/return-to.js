// Convenience check of a `return_to` value on the client. The server applies the same rules and
// is the real check; this only avoids sending an obviously unsafe value.

/**
 * Reads `return_to` from a query string and returns a site-relative path, or `/`.
 * @param {string} search a query string such as `?return_to=/dashboard`
 * @returns {string}
 */
export function returnToFromQuery(search) {
	const value = new URLSearchParams(search).get('return_to');
	if (!value || value.length > 200) return '/';
	if (!value.startsWith('/') || value.startsWith('//') || value.startsWith('/\\')) return '/';
	if (/[\u0000-\u001f\u007f]/.test(value)) return '/';
	try {
		const decoded = decodeURIComponent(value);
		if (decoded.startsWith('//') || decoded.startsWith('/\\')) return '/';
		if (/[\u0000-\u001f\u007f]/.test(decoded)) return '/';
	} catch {
		return '/';
	}
	return value;
}
