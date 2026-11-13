// Familiar requests: saving mute and hide, and asking for one speech bubble per finished run.
// Speech is words only. Nothing here can set a verdict or a mood.
import { asRecord, readEnum, readString } from '../contract';
import { requestJson, type RequestOptions } from './client';

/** The learner's Familiar choices. They are kept locally even when saving them fails. */
export interface FamiliarPrefs {
  muted: boolean;
  hidden: boolean;
}

/** The result of a speech request. Unavailable means a fixed line, never a server message. */
export type SpeechResult =
  | { kind: 'ready'; label: 'AI hint' | 'Measured summary'; text: string }
  | { kind: 'unavailable'; text: string };

const UNAVAILABLE_TEXT =
  'The speech bubble is unavailable right now. The measured result above is complete.';

const MAX_SPEECH_BYTES = 2000;

/**
 * Lets one speech request through per run. Remounts and reconnects ask again, and the guard refuses.
 * The guard is created once per page, so its memory covers the page's session.
 */
export function createSpeechGuard(): { claim(runId: string): boolean } {
  const claimed = new Set<string>();
  return {
    claim(runId: string): boolean {
      if (claimed.has(runId)) return false;
      claimed.add(runId);
      return true;
    }
  };
}

function parseSpeech(raw: unknown): SpeechResult {
  const record = asRecord(raw, 'speech');
  const source = readEnum(record, 'source', 'speech', [
    'ai',
    'template'
  ] as const);
  const text = readString(record, 'text', 'speech', MAX_SPEECH_BYTES);
  return {
    kind: 'ready',
    label: source === 'ai' ? 'AI hint' : 'Measured summary',
    text
  };
}

/** Asks for the speech bubble of one finished run. Any failure becomes the fixed unavailable line. */
export async function requestSpeech(
  runId: string,
  options: RequestOptions = {}
): Promise<SpeechResult> {
  try {
    return await requestJson(
      'POST',
      `/api/runs/${encodeURIComponent(runId)}/familiar-hint`,
      parseSpeech,
      options
    );
  } catch {
    return { kind: 'unavailable', text: UNAVAILABLE_TEXT };
  }
}

/** Saves the mute and hide choices. A failed save reports `synced: false` and keeps the local choice. */
export async function savePreferences(
  prefs: FamiliarPrefs,
  options: RequestOptions = {}
): Promise<{ synced: boolean }> {
  try {
    await requestJson(
      'PATCH',
      '/api/me/familiar',
      (raw) => asRecord(raw, 'prefs'),
      {
        ...options,
        body: prefs
      }
    );
    return { synced: true };
  } catch {
    return { synced: false };
  }
}
