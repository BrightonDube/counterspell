// The workspace's run controller. It starts a run through the real API, watches its public events, and
// loads the authoritative read model when the run finishes. It never invents a result. Every callback
// checks a generation number, so a run that ends after the learner has moved on cannot update the page.
import { ApiRequestError } from '../api/client';
import type { Json } from '../contract';
import {
  getRun,
  startGhostRunAction,
  startRunAction,
  type GhostRunInput,
  type RunMode,
  type StartRunAction,
  type StartRunInput
} from '../runs/api';
import { initialRunState, type RunStreamState } from '../runs/reducer';
import { watchRun, type RunWatch, type WatchOptions } from '../runs/stream';
import type { RunView } from '../runs/types';

/** A Run or Submit request from the editor, carrying the source as it is in the editor right now. */
export interface RunRequest {
  lang: string;
  code: string;
  mode: RunMode;
  /** Marks a Rerun-ghost request; the shell supplies the stored ghost input. */
  ghost?: true;
}

export type SessionPhase =
  'idle' | 'starting' | 'streaming' | 'loading' | 'finished' | 'failed';

export interface RunSessionSnapshot {
  readonly phase: SessionPhase;
  /** The mode of the run in progress or the last one started. */
  readonly mode?: RunMode;
  readonly runId?: string;
  /** The live stream state for the run in progress. */
  readonly stream?: RunStreamState;
  /** The authoritative read model for the last finished run. */
  readonly view?: RunView;
  /** A safe, fixed or server-provided message. Never a stack trace or internal error text. */
  readonly error?: string;
}

/** The side effects the session uses. Tests replace them with fakes. */
export interface RunSessionDeps {
  startAction(input: StartRunInput): StartRunAction;
  /** Posts a custom ghost run. Required only for Rerun-ghost requests. */
  ghostAction?(input: GhostRunInput): StartRunAction;
  fetchRun(runId: string, options: { signal: AbortSignal }): Promise<RunView>;
  watch(options: WatchOptions): RunWatch;
}

/** A start request. `ghostInput` marks a custom ghost run carrying the stored input. */
export interface SessionStartInput extends StartRunInput {
  ghostInput?: Json;
}

export interface RunSession {
  /** Starts one run. Returns false when a run is already in progress, so a double click is ignored. */
  start(input: SessionStartInput): boolean;
  /** Stops watching and ignores anything the old run sends later. Used when the learner leaves. */
  cancel(): void;
  snapshot(): RunSessionSnapshot;
}

/** The production dependencies: the real run API and the real stream client. */
export const browserRunSessionDeps: RunSessionDeps = {
  startAction: startRunAction,
  ghostAction: startGhostRunAction,
  fetchRun: (runId, options) => getRun(runId, options),
  watch: watchRun
};

const FALLBACK_ERROR =
  'The result could not be loaded. Reload the page to try again.';

/** Only the API client's own safe messages reach the page. Anything else gets a fixed line. */
function safeMessage(error: unknown): string {
  return error instanceof ApiRequestError ? error.message : FALLBACK_ERROR;
}

function isBusy(phase: SessionPhase): boolean {
  return phase === 'starting' || phase === 'streaming' || phase === 'loading';
}

/** Creates a session. `onChange` receives every new snapshot, in order. */
export function createRunSession(
  deps: RunSessionDeps,
  onChange: (snapshot: RunSessionSnapshot) => void
): RunSession {
  let current: RunSessionSnapshot = { phase: 'idle' };
  let generation = 0;
  let watch: RunWatch | undefined;
  let pending: AbortController | undefined;

  function publish(next: RunSessionSnapshot): void {
    current = next;
    onChange(next);
  }

  function stopWatch(): void {
    watch?.close();
    watch = undefined;
  }

  /** Fetches the read model for a run that has ended, then publishes it as the finished result. */
  function loadResult(
    token: number,
    runId: string,
    stream: RunStreamState
  ): void {
    publish({ ...current, phase: 'loading', runId, stream, error: undefined });
    pending = new AbortController();
    deps
      .fetchRun(runId, { signal: pending.signal })
      .then((view) => {
        if (token !== generation) return;
        publish({ phase: 'finished', mode: current.mode, runId, stream, view });
      })
      .catch((error: unknown) => {
        if (token !== generation) return;
        // The stream's own terminal event, when there is one, stays on the page.
        publish({
          phase: 'finished',
          mode: current.mode,
          runId,
          stream,
          error: safeMessage(error)
        });
      });
  }

  function watchStream(token: number, runId: string): void {
    watch = deps.watch({
      runId,
      onState: (stream) => {
        if (token !== generation) return;
        publish({ ...current, stream });
      },
      onDone: (stream) => {
        if (token !== generation) return;
        stopWatch();
        loadResult(token, runId, stream);
      },
      onClosed: (stream) => {
        if (token !== generation) return;
        stopWatch();
        loadResult(token, runId, stream);
      },
      onFailure: () => {
        if (token !== generation) return;
        stopWatch();
        loadResult(token, runId, current.stream ?? initialRunState(runId));
      }
    });
  }

  function start(input: SessionStartInput): boolean {
    if (isBusy(current.phase)) return false;
    const action =
      input.ghostInput !== undefined
        ? deps.ghostAction?.({
            problem: input.problem,
            lang: input.lang,
            code: input.code,
            ghost: { input: input.ghostInput }
          })
        : deps.startAction(input);
    if (!action) return false;
    generation += 1;
    const token = generation;
    stopWatch();
    pending?.abort();
    const controller = new AbortController();
    pending = controller;
    // The last finished result stays visible until the replacement has its own terminal result.
    // The prior stream is captured now so a refused start can restore the autopsy.
    const priorStream = current.stream;
    publish({ phase: 'starting', mode: input.mode, view: current.view });

    action
      .send({ signal: controller.signal })
      .then((runId) => {
        if (token !== generation) return;
        publish({
          phase: 'streaming',
          mode: input.mode,
          runId,
          view: current.view,
          stream: initialRunState(runId)
        });
        watchStream(token, runId);
      })
      .catch((error: unknown) => {
        if (token !== generation) return;
        publish({
          phase: 'failed',
          mode: input.mode,
          view: current.view,
          stream: priorStream,
          error: safeMessage(error)
        });
      });
    return true;
  }

  function cancel(): void {
    generation += 1;
    stopWatch();
    pending?.abort();
    pending = undefined;
    publish({ phase: 'idle', view: current.view });
  }

  return {
    start,
    cancel,
    snapshot: () => current
  };
}
