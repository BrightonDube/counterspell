import { describe, expect, it } from 'vitest';
import { ContractError } from '../contract';
import type { RunStreamState } from './reducer';
import { watchRun, type EventSourceLike } from './stream';

/** A fake EventSource. Tests push messages and failures into it, and it never opens a socket. */
class FakeSource implements EventSourceLike {
  readyState = 0;
  onerror: ((event: unknown) => void) | null = null;
  closed = false;
  private readonly listeners = new Map<
    string,
    Array<(event: { data: string }) => void>
  >();

  constructor(readonly url: string) {}

  addEventListener(
    type: string,
    listener: (event: { data: string }) => void
  ): void {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), listener]);
  }

  close(): void {
    this.closed = true;
    this.readyState = 2;
  }

  emit(type: string, seq: number, payload: Record<string, unknown>): void {
    const data = JSON.stringify({ seq, attempt: 1, payload });
    for (const listener of this.listeners.get(type) ?? []) listener({ data });
  }

  fail(): void {
    this.readyState = 2;
    this.onerror?.({});
  }
}

function setup(maxReconnects?: number) {
  const sources: FakeSource[] = [];
  const states: RunStreamState[] = [];
  const done: RunStreamState[] = [];
  const closed: RunStreamState[] = [];
  const failures: unknown[] = [];
  const watch = watchRun({
    runId: 'run-1',
    maxReconnects,
    createEventSource: (url) => {
      const source = new FakeSource(url);
      sources.push(source);
      return source;
    },
    onState: (state) => states.push(state),
    onDone: (state) => done.push(state),
    onClosed: (state) => closed.push(state),
    onFailure: (error) => failures.push(error)
  });
  return { watch, sources, states, done, closed, failures };
}

const compiling = {};
const passRow = { sample: false, ordinal: 1, verdict: 'pass', time_ms: 12 };
const doneOk = { verdict: 'accepted', passed: 1, total: 1, score: 100 };

describe('watchRun', () => {
  it('opens the stream without a cursor on the first connect', () => {
    const { sources } = setup();
    expect(sources.map((source) => source.url)).toEqual([
      '/api/runs/run-1/events'
    ]);
  });

  it('applies events in order and closes the stream after the done event', () => {
    const { sources, states, done } = setup();
    const [source] = sources;
    source.emit('compiling', 1, compiling);
    source.emit('test', 2, passRow);
    source.emit('done', 3, doneOk);

    // The terminal state goes to onDone, so onState sees only the events before it.
    expect(states.map((state) => state.lastSeq)).toEqual([1, 2]);
    expect(done).toHaveLength(1);
    expect(done[0].lastSeq).toBe(3);
    expect(done[0].terminal).toEqual({
      kind: 'done',
      verdict: 'accepted',
      passed: 1,
      total: 1,
      score: 100
    });
    expect(source.closed).toBe(true);
  });

  it('does not report a duplicate event as a state change', () => {
    const { sources, states } = setup();
    sources[0].emit('compiling', 1, compiling);
    sources[0].emit('compiling', 1, compiling);
    expect(states).toHaveLength(1);
  });

  it('reopens from the last applied sequence after a gap', () => {
    const { sources, done } = setup();
    sources[0].emit('compiling', 1, compiling);
    sources[0].emit('test', 3, passRow);

    expect(sources).toHaveLength(2);
    expect(sources[1].url).toBe('/api/runs/run-1/events?last_event_id=1');
    expect(sources[0].closed).toBe(true);

    sources[1].emit('test', 2, passRow);
    sources[1].emit('done', 3, doneOk);
    expect(done).toHaveLength(1);
    expect(done[0].lastSeq).toBe(3);
  });

  it('reports a malformed event with a contract error and stops', () => {
    const { sources, failures, states } = setup();
    sources[0].emit('compiling', 1, compiling);
    sources[0].emit('test', 2, { sample: 'yes' });

    expect(failures).toHaveLength(1);
    expect(failures[0]).toBeInstanceOf(ContractError);
    expect(sources[0].closed).toBe(true);
    expect(states).toHaveLength(1);
  });

  it('calls onClosed with the last state when the server closes without a terminal event', () => {
    const { sources, closed, done } = setup();
    sources[0].emit('compiling', 1, compiling);
    sources[0].fail();

    expect(closed).toHaveLength(1);
    expect(closed[0].lastSeq).toBe(1);
    expect(done).toHaveLength(0);
  });

  it('gives up with a reconnect_limit failure after too many gaps', () => {
    const { sources, failures } = setup(1);
    sources[0].emit('test', 2, passRow);
    expect(sources).toHaveLength(2);

    sources[1].emit('test', 2, passRow);
    expect(failures).toHaveLength(1);
    expect(failures[0]).toMatchObject({ code: 'reconnect_limit' });
    expect(sources[1].closed).toBe(true);
  });

  it('ignores messages that arrive after the caller closes the watch', () => {
    const { watch, sources, states } = setup();
    watch.close();
    sources[0].emit('compiling', 1, compiling);
    expect(states).toHaveLength(0);
    expect(sources[0].closed).toBe(true);
  });
});
