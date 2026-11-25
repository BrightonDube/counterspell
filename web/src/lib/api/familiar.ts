// Familiar requests: saving mute and hide, and asking for one speech bubble per finished run.
// Speech is words only. Nothing here can set a verdict or a mood.
import {
  asRecord,
  ContractError,
  readBool,
  readEnum,
  readString
} from '../contract';
import { requestJson, type RequestOptions } from './client';

/** Whether the learner wants a speech bubble. `off` stops the page asking the server for one. */
export type SpeechPreference = 'auto' | 'off';

/** The learner's Familiar choices. They are kept locally even when saving them fails. */
export interface FamiliarPrefs {
  muted: boolean;
  hidden: boolean;
  /** Read from the server. The page never sends it back with a mute or hide change. */
  speech?: SpeechPreference;
}

/** The result of a speech request. Unavailable means a fixed line, never a server message. */
export type SpeechResult =
  | {
      kind: 'ready';
      label: 'AI hint' | 'Measured summary';
      text: string;
      /** True when the learner's code went to an AI provider to write this text. */
      sentToProvider?: boolean;
    }
  | { kind: 'unavailable'; text: string };

/** The Familiar's own API calls. The workspace injects fakes in tests. */
export interface FamiliarDeps {
  loadPreferences(): Promise<FamiliarPrefs | null>;
  /** The server merges named fields into the stored prefs, so saves send only
   * the fields the learner actually changed. */
  savePreferences(prefs: Partial<FamiliarPrefs>): Promise<{ synced: boolean }>;
  requestSpeech(runId: string): Promise<SpeechResult>;
}

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
  const text = readString(record, 'body', 'speech', MAX_SPEECH_BYTES);
  const sentToProvider = readBool(record, 'sent_to_provider', 'speech');
  return {
    kind: 'ready',
    label: source === 'ai' ? 'AI hint' : 'Measured summary',
    text,
    sentToProvider
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
  prefs: Partial<FamiliarPrefs>,
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

/** Reads the saved mute and hide choices. Returns null when the request fails, so the page keeps its defaults. */
export async function loadPreferences(
  options: RequestOptions = {}
): Promise<FamiliarPrefs | null> {
  try {
    return await requestJson(
      'GET',
      '/api/me/familiar',
      (raw) => {
        const record = asRecord(raw, 'prefs');
        if (
          typeof record.muted !== 'boolean' ||
          typeof record.hidden !== 'boolean'
        ) {
          throw new ContractError(
            'prefs',
            'expected muted and hidden to be booleans'
          );
        }
        const prefs: FamiliarPrefs = {
          muted: record.muted,
          hidden: record.hidden
        };
        if (record.speech === 'auto' || record.speech === 'off') {
          prefs.speech = record.speech;
        }
        return prefs;
      },
      options
    );
  } catch {
    return null;
  }
}
