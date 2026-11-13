// Authored hint requests. The server reveals one rung at a time, and the reply must name the same rung.
import { asRecord, ContractError, readInt, readString } from '../contract';
import { requestJson, type RequestOptions } from './client';

export interface AuthoredHint {
  rung: number;
  text: string;
}

const MAX_HINT_BYTES = 4000;
const MAX_RUNG = 10;

/** Asks the server for one authored hint rung for a submission. */
export function requestAuthoredHint(
  submissionId: string,
  rung: number,
  options: RequestOptions = {}
): Promise<AuthoredHint> {
  return requestJson(
    'POST',
    '/api/hints',
    (raw) => {
      const record = asRecord(raw, 'hint');
      const replied = readInt(record, 'rung', 'hint', 1, MAX_RUNG);
      if (replied !== rung) {
        throw new ContractError(
          'hint.rung',
          'the reply is for a different rung'
        );
      }
      return {
        rung: replied,
        text: readString(record, 'text', 'hint', MAX_HINT_BYTES)
      };
    },
    { ...options, body: { submission_id: submissionId, rung } }
  );
}
