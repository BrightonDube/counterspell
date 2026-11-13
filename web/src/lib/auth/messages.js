// Plain sentences for each error code the auth API returns. The UI shows these, never raw codes.
/** @type {Readonly<Record<string, string>>} */
export const ERROR_MESSAGES = Object.freeze({
  invalid_email: 'Enter a valid email address.',
  weak_password: 'Choose a password of 10 to 128 characters.',
  invalid_json: 'The request could not be read. Please try again.',
  confirmation_mismatch: 'The two passwords do not match.',
  invalid_or_expired_token:
    'This link has been used or has expired. Request a new one.',
  unauthenticated: 'Sign in to continue.',
  invalid_credentials: 'The email or password is not correct.',
  csrf_origin_mismatch: 'This request did not come from Counterspell.',
  demo_account_restricted: 'The demo account cannot do this.',
  provider_disabled: 'This sign-in method is not available.',
  email_already_set: 'This account already has an email address.',
  email_in_use: 'That email address belongs to another account.',
  account_exists:
    'An account with that address already exists. Sign in instead.',
  runs_active: 'Wait for your running submissions to finish first.',
  body_too_large: 'That request is too large.',
  unsupported_media_type: 'The request must be JSON.',
  rate_limited: 'Too many attempts. Wait a few minutes and try again.',
  email_unavailable:
    'Email sign-in is paused right now. Try again later or use a password.',
  busy: 'The server is busy. Try again in a moment.',
  internal_error: 'Something went wrong on our side. Try again later.'
});

/**
 * The plain sentence for an error code. Unknown codes get the generic internal sentence.
 * @param {string} code
 * @returns {string}
 */
export function messageFor(code) {
  return ERROR_MESSAGES[code] ?? ERROR_MESSAGES.internal_error;
}

/**
 * The plain sentence for the `error` code in a query string, or `null` when there is none.
 * @param {string} search a query string such as `?error=invalid_credentials`
 * @returns {string | null}
 */
export function errorFromQuery(search) {
  const code = new URLSearchParams(search).get('error');
  return code ? messageFor(code) : null;
}

/** The providers the account page can confirm as linked, with their display names. */
const LINKED_PROVIDERS = Object.freeze({ github: 'GitHub', google: 'Google' });

/**
 * The confirmation sentence for the `linked` provider in a query string, or `null`. Only the
 * providers listed above are accepted, so the page never shows text taken from the URL.
 * @param {string} search a query string such as `?linked=github`
 * @returns {string | null}
 */
export function linkedMessage(search) {
  const provider = new URLSearchParams(search).get('linked') ?? '';
  if (!Object.hasOwn(LINKED_PROVIDERS, provider)) return null;
  const name =
    LINKED_PROVIDERS[/** @type {keyof typeof LINKED_PROVIDERS} */ (provider)];
  return `${name} is now linked to your account.`;
}
