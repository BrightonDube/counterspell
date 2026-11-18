// Tests for the workspace run session (ide-workspace T13). The fakes stand in for the network and the
// live stream, so no test opens a connection. Each test names the break it catches.
import { describe, expect, it, vi } from 'vitest';
import { ApiRequestError } from '../api/client';
import { initialRunState, type RunStreamState } from '../runs/reducer';
import { StreamFailure, type WatchOptions } from '../runs/stream';
import { parseRunView, type RunView } from '../runs/types';
import {
  createRunSession,
  type RunSessionDeps,
  type RunSessionSnapshot
} from './run-session';

const RUN_ID = '6f1d2c1e-0000-4000-8000-000000000001';
const input = {
  problem: 'two-sum',
  lang: 'python',
  code: 'x = 1',
  mode: 'submit' as const
};

function readModel(runId: string, verdict: string): RunView {
  return parseRunView({
    run_id: runId,
    problem: 'two-sum',
    lang: 'python',
    mode: 'submit',
    status: 'completed',
    terminal: true,
    verdict,
    attempt: 1,
    created_at: '2026-11-15T10:00:00Z',
    last_seq: 3,
    tests: []
  });
}

function terminalStream(runId: string): RunStreamState {
  return {
    ...initialRunState(runId),
    lastSeq: 3,
    status: 'completed',
    terminal: {
      kind: 'done',
      verdict: 'accepted',
      passed: 3,
      total: 3,
      score: 100
    }
  };
}

interface Harness {
  deps: RunSessionDeps;
  watches: WatchOptions[];
  closeWatch: ReturnType<typeof vi.fn>;
  startSend: ReturnType<typeof vi.fn>;
  fetchRun: ReturnType<typeof vi.fn>;
  snapshots: RunSessionSnapshot[];
}

function harness(overrides: Partial<RunSessionDeps> = {}): Harness {
  const watches: WatchOptions[] = [];
  const closeWatch = vi.fn();
  const startSend = vi.fn(() => Promise.resolve(RUN_ID));
  const fetchRun = vi.fn(() => Promise.resolve(readModel(RUN_ID, 'accepted')));
  const deps: RunSessionDeps = {
    startAction: () => ({ key: 'key-1', send: startSend }),
    fetchRun,
    watch: (options) => {
      watches.push(options);
      return { close: closeWatch };
    },
    ...overrides
  };
  return { deps, watches, closeWatch, startSend, fetchRun, snapshots: [] };
}

/** Flushes pending promise callbacks so the session reaches its next state. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

describe('createRunSession', () => {
  // Breaks if a Run click sends nothing, or sends it without the watch that follows the new run id.
  it('starts one run and watches the run id it was given', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    expect(h.startSend).toHaveBeenCalledTimes(1);
    expect(h.watches).toHaveLength(1);
    expect(h.watches[0]?.runId).toBe(RUN_ID);
  });

  // Breaks if a double click queues a second run for the same action.
  it('ignores a second start while the first run is in flight', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    session.start(input);
    await settle();
    expect(h.startSend).toHaveBeenCalledTimes(1);
  });

  // Breaks if a rejected start erases the last finished result the learner is still reading.
  it('keeps the previous finished result when a new start is refused', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    h.watches[0]?.onDone(terminalStream(RUN_ID));
    await settle();
    const before = session.snapshot().view;
    expect(before?.verdict).toBe('accepted');

    h.startSend.mockImplementationOnce(() =>
      Promise.reject(
        new ApiRequestError(429, 'rate_limited', 'Too many requests.', 30)
      )
    );
    session.start(input);
    await settle();
    expect(session.snapshot().view).toBe(before);
    expect(session.snapshot().phase).toBe('failed');
    expect(session.snapshot().error).toBe('Too many requests.');
  });

  // Breaks if the terminal result is taken from the stream alone instead of the authoritative read model.
  it('loads the authoritative read model once a terminal event arrives', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    h.watches[0]?.onDone(terminalStream(RUN_ID));
    await settle();
    expect(h.fetchRun).toHaveBeenCalledTimes(1);
    expect(h.fetchRun.mock.calls[0]?.[0]).toBe(RUN_ID);
    expect(session.snapshot().phase).toBe('finished');
    expect(session.snapshot().view?.verdict).toBe('accepted');
  });

  // Breaks if a stream the server closed early leaves the learner with no result.
  it('loads the read model when the stream closes without a terminal event', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    h.watches[0]?.onClosed(initialRunState(RUN_ID));
    await settle();
    expect(h.fetchRun).toHaveBeenCalledTimes(1);
    expect(session.snapshot().phase).toBe('finished');
  });

  // Breaks if a malformed or over-reconnected stream shows a guessed result instead of the server's.
  it('falls back to the read model when the stream fails', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    h.watches[0]?.onFailure(
      new StreamFailure('reconnect_limit', 'The live updates stopped.')
    );
    await settle();
    expect(h.fetchRun).toHaveBeenCalledTimes(1);
    expect(session.snapshot().view?.verdict).toBe('accepted');
  });

  // Breaks if a read-model failure hides the stream's terminal verdict, or leaks internal error text.
  it('keeps the stream verdict when the read model cannot be loaded', async () => {
    const h = harness({
      fetchRun: vi.fn(() =>
        Promise.reject(
          new ApiRequestError(
            500,
            'internal',
            'The server could not finish this request.'
          )
        )
      )
    });
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    h.watches[0]?.onDone(terminalStream(RUN_ID));
    await settle();
    expect(session.snapshot().phase).toBe('finished');
    expect(session.snapshot().stream?.terminal?.kind).toBe('done');
    expect(session.snapshot().error).toBe(
      'The server could not finish this request.'
    );
  });

  // Breaks if a run that finishes after navigation replaces the result of a different problem.
  it('ignores stream events and results after cancel', async () => {
    const h = harness();
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    session.cancel();
    expect(h.closeWatch).toHaveBeenCalled();
    h.watches[0]?.onDone(terminalStream(RUN_ID));
    await settle();
    expect(h.fetchRun).not.toHaveBeenCalled();
    expect(session.snapshot().view).toBeUndefined();
    expect(session.snapshot().phase).toBe('idle');
  });

  // Breaks if a ghost rerun is posted to the public endpoint, where the stored
  // ghost input is silently ignored and a fresh hunt runs instead.
  it('starts a ghost rerun through the ghost action with the stored input', async () => {
    const ghostSend = vi.fn(() => Promise.resolve('run-ghost'));
    const ghostAction = vi.fn(() => ({ key: 'gk', send: ghostSend }));
    const h = harness({ ghostAction });
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    const started = session.start({ ...input, ghostInput: [0, 0] });
    await settle();

    expect(started).toBe(true);
    expect(ghostAction).toHaveBeenCalledWith(
      expect.objectContaining({
        problem: 'two-sum',
        lang: 'python',
        ghost: { input: [0, 0] }
      })
    );
    expect(h.startSend).not.toHaveBeenCalled();
    expect(h.watches[0]?.runId).toBe('run-ghost');
  });

  // Breaks if a refused ghost rerun throws away the failed run's autopsy, which
  // is the whole reason the learner asked to rerun the ghost.
  it('keeps the stream and finished view when the ghost start is refused', async () => {
    const failed = new ApiRequestError(
      422,
      'validation_failed',
      'The stored ghost is no longer valid.'
    );
    const ghostAction = vi.fn(() => ({
      key: 'gk',
      send: () => Promise.reject(failed)
    }));
    const h = harness({ ghostAction });
    const session = createRunSession(h.deps, (s) => h.snapshots.push(s));
    session.start(input);
    await settle();
    const finishedStream = terminalStream(RUN_ID);
    h.watches[0]?.onDone(finishedStream);
    await settle();
    const priorView = session.snapshot().view;

    session.start({ ...input, ghostInput: [0, 0] });
    await settle();

    expect(session.snapshot().phase).toBe('failed');
    expect(session.snapshot().error).toBe(
      'The stored ghost is no longer valid.'
    );
    expect(session.snapshot().view).toBe(priorView);
    expect(session.snapshot().stream).toBe(finishedStream);
  });
});
