// The delete-account confirmation rule (auth T36, requirement R17.2). The button stays disabled
// until the typed text equals the account's handle exactly. The server checks the same rule again.

/**
 * True only when the typed text is exactly the account's handle. Case and spaces must match.
 * @param {string | undefined} typed what the user typed into the confirmation box
 * @param {string | undefined} handle the account's own handle
 * @returns {boolean}
 */
export function confirmationMatches(typed, handle) {
  if (typeof typed !== 'string' || typeof handle !== 'string') return false;
  if (handle.length === 0) return false;
  return typed === handle;
}
