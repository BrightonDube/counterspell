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
  if (
    !value.startsWith('/') ||
    value.startsWith('//') ||
    value.startsWith('/\\')
  )
    return '/';
  if (hasControlCharacter(value)) return '/';
  try {
    const decoded = decodeURIComponent(value);
    if (decoded.startsWith('//') || decoded.startsWith('/\\')) return '/';
    if (hasControlCharacter(decoded)) return '/';
  } catch {
    return '/';
  }
  return value;
}

/**
 * True when the text has a control character (code 0 to 31, or 127).
 * @param {string} text
 */
function hasControlCharacter(text) {
  for (const character of text) {
    const code = character.codePointAt(0) ?? 0;
    if (code < 32 || code === 127) return true;
  }
  return false;
}
