// Pure event reducer for one run. It has no UI or network code. Each public SSE event is applied in
// sequence order: duplicates and stale events are ignored, a skipped sequence number sets `gap`, and
// `restarted` clears the per-attempt rows. Server text stays as plain strings for the view to render.
import type { GhostView, ShrinkStepView } from '../ghost/types';
import type {
  BenchPoint,
  ComplexityFact,
  MeasuredFamiliar
} from '../familiar/types';
import type {
  DonePayload,
  PublicTest,
  RunEventEnvelope,
  RunEventPayload,
  RunStatus
} from './types';

export type ErrorPayload = Extract<RunEventPayload, { kind: 'error' }>;

/** The final outcome of a run: either a done verdict or a public error. */
export type TerminalPayload = DonePayload | ErrorPayload;

export interface RunStreamState {
  readonly runId: string;
  /** Highest sequence number applied. The next expected event is `lastSeq + 1`. */
  readonly lastSeq: number;
  readonly attempt: number;
  readonly status: RunStatus;
  /** Test rows sorted by ordinal. Each ordinal appears once. */
  readonly tests: readonly PublicTest[];
  readonly shrinkSteps: readonly ShrinkStepView[];
  readonly ghost: GhostView | undefined;
  readonly benchPoints: readonly BenchPoint[];
  readonly complexity: ComplexityFact | undefined;
  readonly familiar: MeasuredFamiliar | undefined;
  readonly terminal: TerminalPayload | undefined;
  /** True after an event arrived ahead of `lastSeq + 1`. The caller must reconnect from `lastSeq`. */
  readonly gap: boolean;
}

/** The state before any event has been applied. */
export function initialRunState(runId: string): RunStreamState {
  return {
    runId,
    lastSeq: 0,
    attempt: 1,
    status: 'queued',
    tests: [],
    shrinkSteps: [],
    ghost: undefined,
    benchPoints: [],
    complexity: undefined,
    familiar: undefined,
    terminal: undefined,
    gap: false
  };
}

/**
 * Applies one envelope. Returns the same object when nothing changes, so callers can detect
 * duplicates by identity. Once a terminal event is applied, later events are ignored.
 */
export function applyEvent(
  state: RunStreamState,
  envelope: RunEventEnvelope
): RunStreamState {
  if (state.terminal !== undefined) return state;
  if (envelope.seq <= state.lastSeq) return state;
  if (envelope.seq > state.lastSeq + 1) {
    return state.gap ? state : { ...state, gap: true };
  }
  const base: RunStreamState = {
    ...state,
    lastSeq: envelope.seq,
    attempt: envelope.attempt,
    gap: false
  };
  return applyPayload(base, envelope.payload);
}

function applyPayload(
  state: RunStreamState,
  payload: RunEventPayload
): RunStreamState {
  switch (payload.kind) {
    case 'compiling':
      return { ...state, status: 'compiling' };
    case 'ghost_hunting':
      return { ...state, status: 'ghost_hunting' };
    case 'restarted':
      return {
        ...state,
        status: 'queued',
        tests: [],
        shrinkSteps: [],
        ghost: undefined,
        benchPoints: [],
        complexity: undefined,
        familiar: undefined
      };
    case 'test':
      return {
        ...state,
        status: 'running',
        tests: upsertTest(state.tests, payload.test)
      };
    case 'shrink_step':
      return { ...state, shrinkSteps: [...state.shrinkSteps, payload.step] };
    case 'ghost':
      return { ...state, ghost: payload.ghost };
    case 'bench_point':
      return { ...state, benchPoints: [...state.benchPoints, payload.point] };
    case 'complexity':
      return { ...state, complexity: payload.fact };
    case 'familiar':
      return { ...state, familiar: payload.familiar };
    case 'done':
      return { ...state, status: 'completed', terminal: payload };
    case 'error':
      return {
        ...state,
        status: payload.code === 'internal_error' ? 'internal_error' : 'failed',
        terminal: payload
      };
  }
}

/** Replaces the row with the same ordinal, or inserts it, keeping rows in ordinal order. */
function upsertTest(
  tests: readonly PublicTest[],
  test: PublicTest
): readonly PublicTest[] {
  const rest = tests.filter((row) => row.ordinal !== test.ordinal);
  return [...rest, test].sort((a, b) => a.ordinal - b.ordinal);
}
