// Same-origin JSON requests for the workspace. Every failure becomes an ApiRequestError with a stable
// code and a fixed or server-safe message. Raw bodies, stack traces and SQL never reach the UI.
import { parseApiError } from './types';

/** A fetch-compatible function. Tests pass a fake, so no test calls the network. */
export type FetchLike = (
  input: string,
  init?: RequestInit
) => Promise<Response>;

export interface RequestOptions {
  /** Cancels the request, for example when the learner opens another problem. */
  signal?: AbortSignal;
  /** How long to wait for the server before failing with the `timeout` code. */
  timeoutMs?: number;
  /** Replaces `fetch`. Used by tests and by any caller that needs a custom transport. */
  fetchImpl?: FetchLike;
}

/** A failed request. `code` is stable and `message` is safe to show a person. */
export class ApiRequestError extends Error {
  readonly status: number;
  readonly code: string;
  readonly retryAfterSeconds?: number;

  constructor(
    status: number,
    code: string,
    message: string,
    retryAfterSeconds?: number
  ) {
    super(message);
    this.name = 'ApiRequestError';
    this.status = status;
    this.code = code;
    if (retryAfterSeconds !== undefined)
      this.retryAfterSeconds = retryAfterSeconds;
  }
}

const DEFAULT_TIMEOUT_MS = 10_000;
const MALFORMED_MESSAGE = 'The server sent a response this page cannot read.';
const SERVER_MESSAGE =
  'The server could not finish this request. Try again in a moment.';
const NETWORK_MESSAGE =
  'The connection to the server failed. Check your connection and try again.';
const TIMEOUT_MESSAGE = 'The server took too long to answer. Try again.';
const ABORTED_MESSAGE = 'The request was cancelled.';

const FALLBACK_CODES: Record<number, { code: string; message: string }> = {
  401: { code: 'unauthorized', message: 'Sign in to continue.' },
  403: { code: 'forbidden', message: 'You cannot open this.' },
  404: { code: 'not_found', message: 'That item was not found.' },
  429: {
    code: 'rate_limited',
    message: 'Too many requests. Wait a moment and try again.'
  }
};

/** Builds the failure for a non-success response. Server messages are shown only for 4xx responses. */
function failureFromResponse(status: number, text: string): ApiRequestError {
  let envelope: ReturnType<typeof parseApiError> | undefined;
  try {
    envelope = parseApiError(JSON.parse(text));
  } catch {
    envelope = undefined;
  }
  if (status >= 500) {
    return new ApiRequestError(
      status,
      envelope?.code ?? 'internal',
      SERVER_MESSAGE
    );
  }
  if (envelope) {
    return new ApiRequestError(
      status,
      envelope.code,
      envelope.message,
      envelope.retryAfterSeconds
    );
  }
  const fallback = FALLBACK_CODES[status] ?? {
    code: 'bad_request',
    message: 'This request was refused.'
  };
  return new ApiRequestError(status, fallback.code, fallback.message);
}

export type HttpMethod = 'GET' | 'POST' | 'PATCH';

/** Options for a JSON request. `body` is sent as JSON when it is not undefined. */
export interface JsonRequestOptions extends RequestOptions {
  body?: unknown;
  /** Extra request headers, for example `Idempotency-Key`. */
  headers?: Record<string, string>;
}

/** Sends a same-origin JSON request and parses the response with `parse`. */
export function requestJson<T>(
  method: HttpMethod,
  path: string,
  parse: (raw: unknown) => T,
  options: JsonRequestOptions = {}
): Promise<T> {
  return sendRequest(method, path, parse, options);
}

/**
 * Sends a same-origin GET and parses the JSON body with `parse`.
 * Throws ApiRequestError for every failure, including malformed JSON and contract violations.
 */
export function getJson<T>(
  path: string,
  parse: (raw: unknown) => T,
  options: RequestOptions = {}
): Promise<T> {
  return sendRequest('GET', path, parse, options);
}

function requestHeaders(options: JsonRequestOptions): Record<string, string> {
  const headers: Record<string, string> = {
    accept: 'application/json',
    ...options.headers
  };
  if (options.body !== undefined) headers['content-type'] = 'application/json';
  return headers;
}

async function sendRequest<T>(
  method: HttpMethod,
  path: string,
  parse: (raw: unknown) => T,
  options: JsonRequestOptions
): Promise<T> {
  const fetchImpl = options.fetchImpl ?? fetch;
  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const controller = new AbortController();
  let timedOut = false;
  const timer = setTimeout(() => {
    timedOut = true;
    controller.abort();
  }, timeoutMs);
  const onCallerAbort = () => controller.abort();
  if (options.signal?.aborted) controller.abort();
  options.signal?.addEventListener('abort', onCallerAbort, { once: true });

  let status: number;
  let text: string;
  try {
    const response = await fetchImpl(path, {
      method,
      credentials: 'same-origin',
      headers: requestHeaders(options),
      body:
        options.body === undefined ? undefined : JSON.stringify(options.body),
      signal: controller.signal
    });
    status = response.status;
    text = await response.text();
  } catch {
    if (options.signal?.aborted)
      throw new ApiRequestError(0, 'aborted', ABORTED_MESSAGE);
    if (timedOut) throw new ApiRequestError(0, 'timeout', TIMEOUT_MESSAGE);
    throw new ApiRequestError(0, 'network', NETWORK_MESSAGE);
  } finally {
    clearTimeout(timer);
    options.signal?.removeEventListener('abort', onCallerAbort);
  }

  if (status < 200 || status > 299) {
    throw failureFromResponse(status, text);
  }
  let decoded: unknown;
  try {
    decoded = JSON.parse(text);
  } catch {
    throw new ApiRequestError(status, 'malformed_response', MALFORMED_MESSAGE);
  }
  try {
    return parse(decoded);
  } catch {
    throw new ApiRequestError(status, 'malformed_response', MALFORMED_MESSAGE);
  }
}
