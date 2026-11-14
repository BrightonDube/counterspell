import { describe, expect, it } from 'vitest';
import { ApiRequestError } from '../api/client';
import type { FetchLike } from '../api/client';
import { getRun, startRun, startRunAction } from './api';

function fakeFetch(
  status: number,
  body: unknown,
  calls: Array<{ path: string; init?: RequestInit }>
): FetchLike {
  return async (path, init) => {
    calls.push({ path, init });
    return new Response(JSON.stringify(body), {
      status,
      headers: { 'content-type': 'application/json' }
    });
  };
}

const runBody = {
  run_id: 'run-1',
  problem: 'two-sum',
  lang: 'python',
  mode: 'run',
  status: 'queued',
  terminal: false,
  attempt: 1,
  created_at: '2026-11-13T10:00:00Z',
  last_seq: 0,
  tests: []
};

describe('startRun', () => {
  it('posts the source with the idempotency key and returns the new run id', async () => {
    const calls: Array<{ path: string; init?: RequestInit }> = [];
    const runId = await startRun(
      { problem: 'two-sum', lang: 'python', code: 'print(1)', mode: 'run' },
      'key-1',
      { fetchImpl: fakeFetch(202, { run_id: 'run-1' }, calls) }
    );

    expect(runId).toBe('run-1');
    expect(calls[0].path).toBe('/api/runs');
    expect(calls[0].init?.method).toBe('POST');
    expect(new Headers(calls[0].init?.headers).get('Idempotency-Key')).toBe(
      'key-1'
    );
    expect(calls[0].init?.body).toBe(
      '{"problem":"two-sum","lang":"python","code":"print(1)","mode":"run"}'
    );
  });

  it('shows a stable error and does not return a run id when the server refuses the source', async () => {
    const calls: Array<{ path: string; init?: RequestInit }> = [];
    await expect(
      startRun(
        { problem: 'two-sum', lang: 'python', code: 'x', mode: 'submit' },
        'key-2',
        {
          fetchImpl: fakeFetch(
            400,
            {
              code: 'source_too_large',
              message: 'The source is 70000 bytes. The limit is 65536.'
            },
            calls
          )
        }
      )
    ).rejects.toMatchObject({ code: 'source_too_large', status: 400 });
  });
});

describe('startRun rate limits', () => {
  it('keeps the wait time from a 429 so the view can tell the learner when to retry', async () => {
    const error = await startRun(
      { problem: 'two-sum', lang: 'python', code: 'print(1)', mode: 'run' },
      'key-3',
      {
        fetchImpl: fakeFetch(
          429,
          {
            code: 'rate_limited',
            message: 'Too many runs.',
            retry_after_seconds: 30
          },
          []
        )
      }
    ).catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(ApiRequestError);
    expect(error).toMatchObject({
      status: 429,
      code: 'rate_limited',
      retryAfterSeconds: 30
    });
  });
});

describe('getRun', () => {
  it('reads the run read model from the same origin', async () => {
    const calls: Array<{ path: string; init?: RequestInit }> = [];
    const view = await getRun('run-1', {
      fetchImpl: fakeFetch(200, runBody, calls)
    });

    expect(calls[0].path).toBe('/api/runs/run-1');
    expect(view).toMatchObject({
      runId: 'run-1',
      status: 'queued',
      terminal: false,
      lastSeq: 0
    });
  });

  it('rejects a response that does not match the run read model', async () => {
    const error = await getRun('run-1', {
      fetchImpl: fakeFetch(200, { run_id: 'run-1' }, [])
    }).catch((caught: unknown) => caught);
    expect(error).toBeInstanceOf(ApiRequestError);
    expect(error).toMatchObject({ code: 'malformed_response' });
  });
});

describe('startRunAction', () => {
  it('reuses one idempotency key for every retry of the same action', async () => {
    const calls: Array<{ path: string; init?: RequestInit }> = [];
    let attempts = 0;
    const flaky: FetchLike = async (path, init) => {
      calls.push({ path, init });
      attempts += 1;
      if (attempts === 1) {
        return new Response('{}', { status: 500 });
      }
      return new Response(JSON.stringify({ run_id: 'run-7' }), {
        status: 202,
        headers: { 'content-type': 'application/json' }
      });
    };

    const action = startRunAction({
      problem: 'two-sum',
      lang: 'python',
      code: 'print(1)',
      mode: 'run'
    });
    await expect(action.send({ fetchImpl: flaky })).rejects.toBeInstanceOf(
      ApiRequestError
    );
    const runId = await action.send({ fetchImpl: flaky });

    expect(runId).toBe('run-7');
    const keys = calls.map((call) =>
      new Headers(call.init?.headers).get('Idempotency-Key')
    );
    expect(keys).toHaveLength(2);
    expect(keys[0]).toBeTruthy();
    expect(keys[1]).toBe(keys[0]);
  });

  it('uses a fresh idempotency key for each new user action', () => {
    const input = {
      problem: 'two-sum',
      lang: 'python',
      code: 'x',
      mode: 'run' as const
    };
    const first = startRunAction(input);
    const second = startRunAction(input);
    expect(first.key).not.toBe(second.key);
  });
});
