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
