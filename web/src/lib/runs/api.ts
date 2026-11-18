// Same-origin run requests. `startRun` sends one idempotency key per user action, so a retried request
// returns the same run instead of creating a second one. Every response is parsed before it is used.
import { getJson, requestJson, type RequestOptions } from '../api/client';
import { asRecord, readString } from '../contract';
import { parseRunView, type RunView } from './types';

export type RunMode = 'run' | 'submit';

export interface StartRunInput {
  problem: string;
  lang: string;
  code: string;
  mode: RunMode;
}

/** A ghost rerun carries the stored ghost input verbatim. The judge decides what it means. */
export interface GhostRunInput {
  problem: string;
  lang: string;
  code: string;
  ghost: { input: unknown };
}

/** Parses the `202` body of `POST /api/runs`, which carries the new run id. */
function parseStartedRun(raw: unknown): string {
  const record = asRecord(raw, 'started');
  return readString(record, 'run_id', 'started', 64);
}

/**
 * Queues a run and returns its id. Pass the same `idempotencyKey` for every retry of one user action.
 * Failures are ApiRequestError values with the server's stable code.
 */
export function startRun(
  input: StartRunInput,
  idempotencyKey: string,
  options: RequestOptions = {}
): Promise<string> {
  return requestJson('POST', '/api/runs', parseStartedRun, {
    ...options,
    body: input,
    headers: { 'Idempotency-Key': idempotencyKey }
  });
}

/** One user action to start a run. Every `send` reuses the same key, so retries cannot queue a second run. */
export interface StartRunAction {
  /** The idempotency key shared by every attempt of this action. */
  readonly key: string;
  /** Sends the request with this action's key and resolves with the run id. */
  send(options?: RequestOptions): Promise<string>;
}

/** Creates one action (one click on Run or Submit) with a single idempotency key. */
export function startRunAction(input: StartRunInput): StartRunAction {
  const key = crypto.randomUUID();
  return {
    key,
    send: (options = {}) => startRun(input, key, options)
  };
}

/** Queues one custom ghost run. Retries share the key, so a double send cannot queue two. */
export function startGhostRun(
  input: GhostRunInput,
  idempotencyKey: string,
  options: RequestOptions = {}
): Promise<string> {
  return requestJson('POST', '/api/runs/ghost', parseStartedRun, {
    ...options,
    body: input,
    headers: { 'Idempotency-Key': idempotencyKey }
  });
}

/** Creates one Rerun-ghost click with a single idempotency key. */
export function startGhostRunAction(input: GhostRunInput): StartRunAction {
  const key = crypto.randomUUID();
  return {
    key,
    send: (options = {}) => startGhostRun(input, key, options)
  };
}

/** Reads the authoritative run read model, for reloads and for a stream that closed early. */
export function getRun(
  runId: string,
  options: RequestOptions = {}
): Promise<RunView> {
  return getJson(
    `/api/runs/${encodeURIComponent(runId)}`,
    parseRunView,
    options
  );
}
