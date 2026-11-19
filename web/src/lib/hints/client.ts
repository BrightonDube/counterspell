// Hint requests for the IDE (hints spec H15). The server reveals authored ladder rungs and makes AI
// hints on click. Every reply is checked against the request, so a reply for another rung is refused.
import { requestJson, type RequestOptions } from '../api/client';
import {
  asRecord,
  readBool,
  readEnum,
  readInt,
  readOptionalInt,
  readOptionalString,
  readString,
  type RawRecord
} from '../contract';

/** Where a hint came from. `ladder` is authored text; `ai` is generated and labelled. */
export type HintSourceName = 'ladder' | 'ai';

/** The learner's AI quota after a request. */
export interface HintQuota {
  used: number;
  limit: number;
  resetsAt: string;
}

/** One hint from the server, with the facts the panel needs to label it. */
export interface HintReply {
  rung: number;
  source: HintSourceName;
  body: string;
  label: string | null;
  provider: string | null;
  model: string | null;
  sentToProvider: boolean;
  quota: HintQuota | null;
  nextRung: number | null;
}

/** What the learner asked for. */
export interface HintRequest {
  submissionId: string;
  rung: number;
  ai: boolean;
}

const MIN_RUNG = 1;
const MAX_RUNG = 4;
const MAX_HINT_BYTES = 4000;
const MAX_QUOTA = 100_000;

/**
 * Asks the server for one hint rung, or one AI hint for a rung. Throws `ApiRequestError` for any
 * failure, including a reply for a different rung or a reply that breaks the contract.
 */
export function requestHint(
  request: HintRequest,
  options: RequestOptions = {}
): Promise<HintReply> {
  if (
    !Number.isInteger(request.rung) ||
    request.rung < MIN_RUNG ||
    request.rung > MAX_RUNG
  ) {
    return Promise.reject(
      new TypeError(`hint rung must be between ${MIN_RUNG} and ${MAX_RUNG}`)
    );
  }
  return requestJson(
    'POST',
    '/api/hints',
    (raw) => parseReply(raw, request.rung),
    {
      ...options,
      body: {
        submission_id: request.submissionId,
        rung: request.rung,
        ai: request.ai
      }
    }
  );
}

function parseReply(raw: unknown, expectedRung: number): HintReply {
  const record = asRecord(raw, 'hint');
  const rung = readInt(record, 'rung', 'hint', MIN_RUNG, MAX_RUNG);
  if (rung !== expectedRung) {
    throw new RangeError('the reply is for a different rung');
  }
  return {
    rung,
    source: readEnum<HintSourceName>(record, 'source', 'hint', [
      'ladder',
      'ai'
    ]),
    body: readString(record, 'body', 'hint', MAX_HINT_BYTES),
    label: readOptionalString(record, 'label', 'hint', MAX_HINT_BYTES) ?? null,
    provider:
      readOptionalString(record, 'provider', 'hint', MAX_HINT_BYTES) ?? null,
    model: readOptionalString(record, 'model', 'hint', MAX_HINT_BYTES) ?? null,
    sentToProvider: readBool(record, 'sent_to_provider', 'hint'),
    quota: parseQuota(record),
    nextRung:
      readOptionalInt(record, 'next_rung', 'hint', MIN_RUNG, MAX_RUNG) ?? null
  };
}

function parseQuota(record: RawRecord): HintQuota | null {
  const raw = record.quota;
  if (raw === undefined || raw === null) return null;
  const quota = asRecord(raw, 'hint.quota');
  return {
    used: readInt(quota, 'used', 'hint.quota', 0, MAX_QUOTA),
    limit: readInt(quota, 'limit', 'hint.quota', 0, MAX_QUOTA),
    resetsAt: readString(quota, 'resets_at', 'hint.quota', 64)
  };
}
