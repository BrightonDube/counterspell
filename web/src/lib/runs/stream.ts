// Stream client for one run. It wraps EventSource, feeds each public event to the pure reducer, and
// reopens from the last applied sequence after a gap. A stream the server closes before a terminal
// event is handed to the caller, who fetches the authoritative run read model.
import { applyEvent, initialRunState, type RunStreamState } from './reducer';
import { parseEventEnvelope, RUN_EVENT_NAMES } from './types';

/** EventSource.CLOSED. The browser has stopped reconnecting, either because the server said so or after an error. */
const READY_STATE_CLOSED = 2;
const DEFAULT_MAX_RECONNECTS = 5;

/** The part of EventSource this client uses. Tests supply a fake, so no test opens a connection. */
export interface EventSourceLike {
  readonly readyState: number;
  onerror: ((event: unknown) => void) | null;
  addEventListener(
    type: string,
    listener: (event: { data: string }) => void
  ): void;
  close(): void;
}

export type EventSourceFactory = (url: string) => EventSourceLike;

/** A stream-level failure with a stable code. Its message is fixed text, never server output. */
export class StreamFailure extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'StreamFailure';
    this.code = code;
  }
}

export interface WatchOptions {
  runId: string;
  /** Called after each event that changes the state. */
  onState: (state: RunStreamState) => void;
  /** Called once, after the terminal event has been applied and the stream closed. */
  onDone: (state: RunStreamState) => void;
  /** Called when the server closes the stream without a terminal event. Fetch the run read model. */
  onClosed: (state: RunStreamState) => void;
  /** Called with a ContractError for a malformed event, or a StreamFailure after too many gaps. */
  onFailure: (error: unknown) => void;
  /** Overrides how the EventSource is created. Defaults to the browser's EventSource. */
  createEventSource?: EventSourceFactory;
  /** How many gap reconnects are allowed before the watch gives up. */
  maxReconnects?: number;
}

export interface RunWatch {
  /** Closes the stream and ignores any later message. */
  close(): void;
}

function browserEventSource(url: string): EventSourceLike {
  return new globalThis.EventSource(url) as unknown as EventSourceLike;
}

/** Watches a run's public events from sequence 0. Reconnects use `?last_event_id=` after a gap. */
export function watchRun(options: WatchOptions): RunWatch {
  const createSource = options.createEventSource ?? browserEventSource;
  const maxReconnects = options.maxReconnects ?? DEFAULT_MAX_RECONNECTS;
  const base = `/api/runs/${encodeURIComponent(options.runId)}/events`;
  let state: RunStreamState = initialRunState(options.runId);
  let source: EventSourceLike | undefined;
  let reconnects = 0;
  let stopped = false;

  function stop(): void {
    stopped = true;
    source?.close();
    source = undefined;
  }

  function receive(name: string, data: string): void {
    if (stopped) return;
    let next: RunStreamState;
    try {
      next = applyEvent(state, parseEventEnvelope(name, data));
    } catch (error) {
      stop();
      options.onFailure(error);
      return;
    }
    if (next.gap) {
      state = next;
      reopen();
      return;
    }
    if (next === state) return;
    state = next;
    if (state.terminal !== undefined) {
      stop();
      options.onDone(state);
      return;
    }
    options.onState(state);
  }

  function open(): void {
    const cursor = state.lastSeq;
    const url = cursor > 0 ? `${base}?last_event_id=${cursor}` : base;
    const next = createSource(url);
    source = next;
    for (const name of RUN_EVENT_NAMES) {
      next.addEventListener(name, (message) => receive(name, message.data));
    }
    next.onerror = () => onSourceError(next);
  }

  function reopen(): void {
    state = { ...state, gap: false };
    source?.close();
    reconnects += 1;
    if (reconnects > maxReconnects) {
      stop();
      options.onFailure(
        new StreamFailure(
          'reconnect_limit',
          'The live updates stopped after too many reconnects.'
        )
      );
      return;
    }
    open();
  }

  function onSourceError(failed: EventSourceLike): void {
    if (stopped || failed !== source) return;
    if (failed.readyState !== READY_STATE_CLOSED) return;
    // The browser retries on its own while CONNECTING. A CLOSED stream means the server ended it.
    stop();
    options.onClosed(state);
  }

  open();
  return { close: stop };
}
