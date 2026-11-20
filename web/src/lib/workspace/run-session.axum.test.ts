// @vitest-environment node
// Integration: the production run client and session against a real Axum server.
// Nothing here mocks the server — every value asserted was emitted by Axum.
//
// Start the fixture first:
//   DATABASE_URL=postgres://... CS_TEST_PORT=5198 RUN_RATE_USER=1 \
//     RUN_RATE_IP=10000 cargo run -p counterspell-testserver
// RUN_RATE_IP lifts the loopback IP cap so repeated suite runs are refused only
// by the per-user limit the refusal tests exercise.
// Then: AXUM_TEST_URL=http://127.0.0.1:5198 npx vitest run run-session.axum
import { describe, expect, it } from 'vitest';
import { ApiRequestError, type FetchLike } from '../api/client';
import { getRun, startRunAction } from '../runs/api';
import { watchRun, type EventSourceLike } from '../runs/stream';
import { createRunSession, type RunSessionSnapshot } from './run-session';

const BASE = process.env.AXUM_TEST_URL;
const token = (n: number) => `Bearer test-user-${n}`;
// RUN_RATE_USER=1 on the fixture: a user's first POST succeeds, the second in
// the window is refused. Each test draws a fresh user so reruns stay true.
const describeAxum = BASE ? describe : describe.skip;
// The fixture seeds 4096 users; 1-8 are for manual smoke. The rate window is
// 600 s per user, and one suite run draws USERS_PER_RUN users. Users 9-4093
// form 817 slots of 5; the slot advances every second, so a slot comes round
// again only after 817 s, longer than the window. A suite run lasts longer
// than one second, so back-to-back reruns never reuse a spent user.
const USERS_PER_RUN = 5;
const SLOTS = 817;
const firstUser = 9 + (Math.floor(Date.now() / 1000) % SLOTS) * USERS_PER_RUN;
let nextUser = firstUser;
const freshUser = () => nextUser++;

/** Same-origin paths become absolute URLs to the fixture, with the bearer token. */
function axumFetch(user: number): FetchLike {
  return (input, init) =>
    fetch(`${BASE}${input}`, {
      ...init,
      headers: {
        ...(init?.headers as Record<string, string> | undefined),
        authorization: token(user)
      }
    }).catch((error: unknown) => {
      // The client maps every transport failure to one friendly message; keep
      // the real cause visible when this suite fails.
      const cause = (error as { cause?: unknown }).cause;
      console.error(
        'fetch %s %s failed:',
        init?.method ?? 'GET',
        String(input),
        error,
        cause
      );
      throw error;
    });
}

/** Fetch-backed EventSourceLike: real SSE bytes with an Authorization header. */
function sseSource(user: number): (url: string) => EventSourceLike {
  return (url) => {
    const listeners = new Map<string, ((event: { data: string }) => void)[]>();
    const controller = new AbortController();
    let readyState = 0;
    const source: EventSourceLike = {
      get readyState() {
        return readyState;
      },
      onerror: null,
      addEventListener(type, listener) {
        const list = listeners.get(type) ?? [];
        list.push(listener);
        listeners.set(type, list);
      },
      close() {
        controller.abort();
        readyState = 2;
      }
    };
    void (async () => {
      try {
        const response = await fetch(`${BASE}${url}`, {
          headers: {
            authorization: token(user),
            accept: 'text/event-stream'
          },
          signal: controller.signal
        });
        readyState = 1;
        const reader = response.body?.getReader();
        if (!reader) throw new Error('no SSE body');
        const decoder = new TextDecoder();
        let buffer = '';
        for (;;) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          let cut: number;
          while ((cut = buffer.indexOf('\n\n')) !== -1) {
            const frame = buffer.slice(0, cut);
            buffer = buffer.slice(cut + 2);
            let name = 'message';
            const data: string[] = [];
            for (const line of frame.split('\n')) {
              if (line.startsWith('event:')) name = line.slice(6).trim();
              else if (line.startsWith('data:'))
                data.push(line.slice(5).trim());
            }
            for (const listener of listeners.get(name) ?? []) {
              listener({ data: data.join('\n') });
            }
          }
        }
        readyState = 2;
        source.onerror?.(new Event('closed'));
      } catch (error) {
        if (controller.signal.aborted) return;
        readyState = 2;
        source.onerror?.(error);
      }
    })();
    return source;
  };
}

/** Real session deps pointed at the fixture server. */
function sessionDeps(user: number) {
  const fetchImpl = axumFetch(user);
  return {
    startAction: (input: Parameters<typeof startRunAction>[0]) => {
      const action = startRunAction(input);
      return {
        key: action.key,
        send: (options = {}) => action.send({ ...options, fetchImpl })
      };
    },
    fetchRun: (runId: string, options: { signal: AbortSignal }) =>
      getRun(runId, { ...options, fetchImpl }),
    watch: (options: Parameters<typeof watchRun>[0]) =>
      watchRun({ ...options, createEventSource: sseSource(user) })
  };
}

function untilFinished(
  snapshots: RunSessionSnapshot[],
  ready: () => boolean
): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () =>
        reject(
          new Error(
            `no terminal snapshot; last=${snapshots.at(-1)?.phase}; error=${JSON.stringify(snapshots.at(-1)?.error)}`
          )
        ),
      15000
    );
    const tick = setInterval(() => {
      if (ready()) {
        clearInterval(tick);
        clearTimeout(timer);
        resolve();
      }
    }, 20);
  });
}

describeAxum('run session against a real Axum server', () => {
  it('shows a successful sample run from real responses, not mocks', async () => {
    const snapshots: RunSessionSnapshot[] = [];
    const session = createRunSession(sessionDeps(freshUser()), (next) =>
      snapshots.push(next)
    );
    expect(
      session.start({
        problem: 'two-sum',
        lang: 'python',
        code: 'print(1)\n',
        mode: 'run'
      })
    ).toBe(true);

    await untilFinished(
      snapshots,
      () => snapshots.at(-1)?.phase === 'finished'
    );
    const done = snapshots.at(-1);
    if (!done) throw new Error('the session published no snapshot');
    // Every asserted field was emitted by Axum: the 202 id, the streamed
    // states, and the terminal read model's verdict and test rows.
    expect(done.runId).toMatch(/^[0-9a-f-]{36}$/);
    expect(done.view?.status).toBe('completed');
    expect(done.view?.tests).toHaveLength(3);
    expect(done.error).toBeUndefined();
    session.cancel();
    // The first run after a cold fixture start can take several seconds; the
    // test's own wait is 15 s, so the vitest limit must sit above it.
  }, 20000);

  it('a refused submission shows the server message and keeps the prior result', async () => {
    const snapshots: RunSessionSnapshot[] = [];
    const session = createRunSession(sessionDeps(freshUser()), (next) =>
      snapshots.push(next)
    );
    session.start({
      problem: 'two-sum',
      lang: 'python',
      code: 'print(1)\n',
      mode: 'run'
    });
    await untilFinished(
      snapshots,
      () => snapshots.at(-1)?.phase === 'finished'
    );
    const prior = snapshots.at(-1);
    if (!prior) throw new Error('the session published no snapshot');

    session.start({
      problem: 'two-sum',
      lang: 'python',
      code: 'x'.repeat(70_000),
      mode: 'submit'
    });
    await untilFinished(snapshots, () => snapshots.at(-1)?.phase === 'failed');
    const failed = snapshots.at(-1);
    if (!failed) throw new Error('the session published no snapshot');
    // The message came from the server's own error envelope.
    expect(failed.error).toBeTruthy();
    // The earlier finished view survived the refused request.
    expect(failed.view?.status).toBe(prior.view?.status);
    session.cancel();
  });

  it('a rate-limited run reports rate_limited and keeps the prior result', async () => {
    const snapshots: RunSessionSnapshot[] = [];
    const session = createRunSession(sessionDeps(freshUser()), (next) =>
      snapshots.push(next)
    );
    session.start({
      problem: 'two-sum',
      lang: 'python',
      code: 'print(1)\n',
      mode: 'run'
    });
    await untilFinished(
      snapshots,
      () => snapshots.at(-1)?.phase === 'finished'
    );

    // RUN_RATE_USER=1, so this user's second request inside the window is refused.
    session.start({
      problem: 'two-sum',
      lang: 'python',
      code: 'print(2)\n',
      mode: 'run'
    });
    await untilFinished(snapshots, () => snapshots.at(-1)?.phase === 'failed');
    expect(snapshots.at(-1)?.error).toBeTruthy();
    expect(snapshots.at(-1)?.view?.status).toBe('completed');
    session.cancel();
  });

  it('a foreign run id is not found and reveals nothing', async () => {
    const fetchImpl = axumFetch(freshUser());
    const runId = await startRunAction({
      problem: 'two-sum',
      lang: 'python',
      code: 'print(1)\n',
      mode: 'run'
    }).send({ fetchImpl });
    // User 5 cannot read user 4's run; the 404 gives a safe message only.
    const error = await getRun(runId, {
      fetchImpl: axumFetch(freshUser())
    }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiRequestError);
    expect((error as ApiRequestError).status).toBe(404);
    expect((error as ApiRequestError).code).toBe('not_found');
  });
});
