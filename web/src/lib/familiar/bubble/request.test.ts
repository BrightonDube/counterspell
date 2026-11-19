// Tests for the once-per-run speech request (efficiency-familiar T39, R11.8, R11.9).
// Each test names the break it catches. `fetch` is injected, so no test touches the network.
import { describe, expect, it } from 'vitest';
import { createBubbleRequester } from './request';

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' }
  });
}

const aiBody = {
  run_id: 'run-1',
  source: 'ai',
  label: 'AI hint',
  body: 'The nested scan looks quadratic.',
  provider: 'openai',
  model: 'gpt-6-luna',
  sent_to_provider: true,
  reason: null,
  quota: null
};

describe('createBubbleRequester', () => {
  // Breaks if a remount asks the server (and the model) a second time for the same run.
  it('returns the same promise for the same run and calls the server once', async () => {
    let calls = 0;
    const requester = createBubbleRequester(async () => {
      calls += 1;
      return jsonResponse(200, aiBody);
    });

    const first = requester.request('run-1', 'auto');
    const second = requester.request('run-1', 'auto');

    expect(second).toBe(first);
    await first;
    expect(calls).toBe(1);
  });

  // Breaks if a second run reuses the first run's bubble.
  it('asks again for a different run', async () => {
    const seen: string[] = [];
    const requester = createBubbleRequester(async (url) => {
      seen.push(url);
      return jsonResponse(200, aiBody);
    });

    await requester.request('run-1', 'auto');
    await requester.request('run-2', 'auto');

    expect(seen).toEqual([
      '/api/runs/run-1/familiar-hint',
      '/api/runs/run-2/familiar-hint'
    ]);
  });

  // Breaks if "speech off" still sends the learner's code to the server for a bubble.
  it('does not call the server when speech is off', async () => {
    let calls = 0;
    const requester = createBubbleRequester(async () => {
      calls += 1;
      return jsonResponse(200, aiBody);
    });

    const result = await requester.request('run-1', 'off');

    expect(result).toBeNull();
    expect(calls).toBe(0);
  });

  // Breaks if turning speech off for a run poisons the memo, so turning it back on never asks.
  it('still asks once speech is switched back on', async () => {
    let calls = 0;
    const requester = createBubbleRequester(async () => {
      calls += 1;
      return jsonResponse(200, aiBody);
    });

    await requester.request('run-1', 'off');
    await requester.request('run-1', 'auto');

    expect(calls).toBe(1);
  });

  // Breaks if a failure shows the server's text, or retries and spends the learner's daily AI quota again.
  it('turns a failure into the fixed message and does not retry the run', async () => {
    let calls = 0;
    const requester = createBubbleRequester(async () => {
      calls += 1;
      return jsonResponse(500, {
        code: 'internal',
        message: 'db password leaked'
      });
    });

    const first = await requester.request('run-1', 'auto');
    const second = await requester.request('run-1', 'auto');

    expect(first?.kind).toBe('unavailable');
    expect(JSON.stringify(first)).not.toContain('leaked');
    expect(second).toEqual(first);
    expect(calls).toBe(1);
  });
});
