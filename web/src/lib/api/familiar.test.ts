// Tests for the Familiar API adapters and the one-speech-request guard (ide-workspace T20).
// Each test names the break it catches. `fetch` is injected, so no test touches the network.
import { describe, expect, it } from 'vitest';
import { createSpeechGuard, requestSpeech, savePreferences } from './familiar';

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

describe('createSpeechGuard', () => {
  // Breaks if a remount or reconnect asks the model a second time for the same finished run.
  it('allows one speech request per run and refuses repeats', () => {
    const guard = createSpeechGuard();
    expect(guard.claim('run-1')).toBe(true);
    expect(guard.claim('run-1')).toBe(false);
  });

  // Breaks if the guard blocks a different run, so the second submission never gets its speech bubble.
  it('allows a request for a different run', () => {
    const guard = createSpeechGuard();
    guard.claim('run-1');
    expect(guard.claim('run-2')).toBe(true);
  });
});

describe('requestSpeech', () => {
  // Breaks if the AI text is shown without its AI hint label, so learners cannot tell it from measured data.
  it('posts once for the run and labels AI text as an AI hint', async () => {
    const calls: { url: string; method: string | undefined }[] = [];
    const fetchImpl = async (url: string, init?: RequestInit) => {
      calls.push({ url, method: init?.method });
      return jsonResponse(200, {
        source: 'ai',
        text: 'The nested scan drives the growth.'
      });
    };
    const speech = await requestSpeech('run-1', { fetchImpl });
    expect(calls).toEqual([
      { url: '/api/runs/run-1/familiar-hint', method: 'POST' }
    ]);
    expect(speech).toEqual({
      kind: 'ready',
      label: 'AI hint',
      text: 'The nested scan drives the growth.'
    });
  });

  // Breaks if a failed speech request invents a sentence, or shows the server's internal error text.
  it('falls back to a fixed unavailable line when the request fails', async () => {
    const fetchImpl = async () =>
      jsonResponse(502, { code: 'provider', message: 'upstream key expired' });
    const speech = await requestSpeech('run-1', { fetchImpl });
    expect(speech.kind).toBe('unavailable');
    expect(JSON.stringify(speech)).not.toContain('upstream key');
  });
});

describe('savePreferences', () => {
  // Breaks if the mute and hide choices are sent with the wrong method or body, so they never persist.
  it('patches the mute and hide preferences as JSON', async () => {
    let seen: { method?: string; body?: string } = {};
    const fetchImpl = async (_url: string, init?: RequestInit) => {
      seen = { method: init?.method, body: init?.body as string };
      return jsonResponse(200, { muted: true, hidden: false });
    };
    const result = await savePreferences(
      { muted: true, hidden: false },
      { fetchImpl }
    );
    expect(seen.method).toBe('PATCH');
    expect(JSON.parse(seen.body ?? '{}')).toEqual({
      muted: true,
      hidden: false
    });
    expect(result).toEqual({ synced: true });
  });

  // Breaks if a failed save blocks the editor or drops the local choice, when the spec says it must keep it.
  it('reports a pending sync without throwing when the save fails', async () => {
    const fetchImpl = async () => {
      throw new TypeError('network down');
    };
    const result = await savePreferences(
      { muted: true, hidden: true },
      { fetchImpl }
    );
    expect(result).toEqual({ synced: false });
  });
});
