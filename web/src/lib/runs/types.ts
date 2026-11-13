// Browser view models for the run read model and the public SSE envelope. Event names and payload
// visibility follow api-conventions.md and the server's public event types. Hidden test rows carry
// ordinal, verdict and time only, and the parser refuses any hidden row that has more.
import {
  asRecord,
  ContractError,
  DISPLAY_CAP_BYTES,
  readArray,
  readBool,
  readEnum,
  readInt,
  readOptionalString,
  readString,
  type RawRecord
} from '../contract';
import {
  parseBenchPoint,
  parseComplexityFact,
  parseMeasuredFamiliar,
  type BenchPoint,
  type ComplexityFact,
  type MeasuredFamiliar
} from '../familiar/types';
import {
  parseGhost,
  parseShrinkStep,
  type GhostView,
  type ShrinkStepView
} from '../ghost/types';

/** The SSE event names the server may send, in the documented order. */
export const RUN_EVENT_NAMES = [
  'compiling',
  'test',
  'ghost_hunting',
  'shrink_step',
  'ghost',
  'bench_point',
  'complexity',
  'familiar',
  'restarted',
  'done',
  'error'
] as const;
export type RunEventName = (typeof RUN_EVENT_NAMES)[number];

/** Per-test verdicts a judge may report. */
export const TEST_VERDICTS = [
  'pass',
  'wrong_answer',
  'runtime_error',
  'time_limit',
  'memory_limit',
  'output_limit'
] as const;
export type TestVerdict = (typeof TEST_VERDICTS)[number];

/** Run-level verdicts reported on a finished submission. */
export const RUN_VERDICTS = [
  'accepted',
  'wrong_answer',
  'runtime_error',
  'time_limit',
  'memory_limit',
  'output_limit',
  'compile_error'
] as const;
export type RunVerdict = (typeof RUN_VERDICTS)[number];

/** Lifecycle statuses of a run, as the read model reports them. */
export const RUN_STATUSES = [
  'queued',
  'claimed',
  'compiling',
  'running',
  'ghost_hunting',
  'completed',
  'failed',
  'expired',
  'internal_error'
] as const;
export type RunStatus = (typeof RUN_STATUSES)[number];

/** Field names that only a hidden test may never carry. Their presence means the row leaks data. */
const HIDDEN_FORBIDDEN_KEYS = [
  'input',
  'expected',
  'got',
  'stdout',
  'stderr',
  'detail'
] as const;

const MAX_TESTS = 1000;
const MAX_TIME_MS = 3_600_000;
const MAX_SEQ = Number.MAX_SAFE_INTEGER;

export interface HiddenTest {
  sample: false;
  ordinal: number;
  verdict: TestVerdict;
  timeMs: number;
}

export interface SampleTest {
  sample: true;
  ordinal: number;
  verdict: TestVerdict;
  timeMs: number;
  input: string;
  expected: string;
  got: string;
}

export type PublicTest = HiddenTest | SampleTest;

export interface RunView {
  runId: string;
  problem: string;
  lang: string;
  mode: 'run' | 'submit';
  status: RunStatus;
  terminal: boolean;
  verdict?: RunVerdict;
  failureReason?: string;
  attempt: number;
  passed?: number;
  total?: number;
  score?: number;
  createdAt: string;
  startedAt?: string;
  finishedAt?: string;
  lastSeq: number;
  tests: PublicTest[];
}

export type DonePayload = {
  kind: 'done';
  verdict: RunVerdict;
  passed: number;
  total: number;
  score: number;
  compileOutput?: string;
};

export type RunEventPayload =
  | { kind: 'compiling' }
  | { kind: 'ghost_hunting' }
  | { kind: 'restarted'; attempt: number }
  | { kind: 'test'; test: PublicTest }
  | { kind: 'shrink_step'; step: ShrinkStepView }
  | { kind: 'ghost'; ghost: GhostView }
  | { kind: 'bench_point'; point: BenchPoint }
  | { kind: 'complexity'; fact: ComplexityFact }
  | { kind: 'familiar'; familiar: MeasuredFamiliar }
  | DonePayload
  | { kind: 'error'; code: string; message: string };

export interface RunEventEnvelope {
  seq: number;
  attempt: number;
  event: RunEventName;
  payload: RunEventPayload;
}

/** Parses one test row. Hidden rows with any detail field are refused, never trimmed silently. */
export function parsePublicTest(raw: unknown, path = 'test'): PublicTest {
  const record = asRecord(raw, path);
  const common = {
    ordinal: readInt(record, 'ordinal', path, 1, MAX_TESTS),
    verdict: readEnum(record, 'verdict', path, TEST_VERDICTS),
    timeMs: readInt(record, 'time_ms', path, 0, MAX_TIME_MS)
  };
  if (record.sample === true) {
    return {
      sample: true,
      ...common,
      input: readString(record, 'input', path),
      expected: readString(record, 'expected', path),
      got: readString(record, 'got', path)
    };
  }
  if (record.sample !== false) {
    throw new ContractError(`${path}.sample`, 'expected a boolean');
  }
  for (const key of HIDDEN_FORBIDDEN_KEYS) {
    if (key in record) {
      throw new ContractError(
        `${path}.${key}`,
        'hidden test rows carry no detail'
      );
    }
  }
  return { sample: false, ...common };
}

function optionalEnum<T extends string>(
  source: RawRecord,
  key: string,
  path: string,
  allowed: readonly T[]
): T | undefined {
  if (source[key] === undefined || source[key] === null) return undefined;
  return readEnum(source, key, path, allowed);
}

/** Parses `GET /api/runs/{id}`. Sections that the server did not send stay absent. */
export function parseRunView(raw: unknown): RunView {
  const path = 'run';
  const record = asRecord(raw, path);
  const view: RunView = {
    runId: readString(record, 'run_id', path, 64),
    problem: readString(record, 'problem', path, 256),
    lang: readString(record, 'lang', path, 64),
    mode: readEnum(record, 'mode', path, ['run', 'submit'] as const),
    status: readEnum(record, 'status', path, RUN_STATUSES),
    terminal: readBool(record, 'terminal', path),
    attempt: readInt(record, 'attempt', path, 1, 1000),
    createdAt: readString(record, 'created_at', path, 64),
    lastSeq: readInt(record, 'last_seq', path, 0, MAX_SEQ),
    tests: readArray(record, 'tests', path, MAX_TESTS, (value, itemPath) =>
      parsePublicTest(value, itemPath)
    )
  };
  const verdict = optionalEnum(record, 'verdict', path, RUN_VERDICTS);
  if (verdict !== undefined) view.verdict = verdict;
  const failure = readOptionalString(record, 'failure_reason', path, 128);
  if (failure !== undefined) view.failureReason = failure;
  const startedAt = readOptionalString(record, 'started_at', path, 64);
  if (startedAt !== undefined) view.startedAt = startedAt;
  const finishedAt = readOptionalString(record, 'finished_at', path, 64);
  if (finishedAt !== undefined) view.finishedAt = finishedAt;
  for (const [key, target] of [
    ['passed', 'passed'],
    ['total', 'total'],
    ['score', 'score']
  ] as const) {
    if (record[key] !== undefined && record[key] !== null) {
      view[target] = readInt(record, key, path, 0, MAX_TESTS);
    }
  }
  return view;
}

function parseDone(payload: RawRecord, path: string): DonePayload {
  const done: DonePayload = {
    kind: 'done',
    verdict: readEnum(payload, 'verdict', path, RUN_VERDICTS),
    passed: readInt(payload, 'passed', path, 0, MAX_TESTS),
    total: readInt(payload, 'total', path, 0, MAX_TESTS),
    score: readInt(payload, 'score', path, 0, 100)
  };
  const compileOutput = readOptionalString(
    payload,
    'compile_output',
    path,
    8 * 1024
  );
  if (compileOutput !== undefined) done.compileOutput = compileOutput;
  return done;
}

function parsePayload(
  event: RunEventName,
  payload: RawRecord,
  path: string
): RunEventPayload {
  switch (event) {
    case 'compiling':
      return { kind: 'compiling' };
    case 'ghost_hunting':
      return { kind: 'ghost_hunting' };
    case 'restarted':
      return {
        kind: 'restarted',
        attempt: readInt(payload, 'attempt', path, 1, 1000)
      };
    case 'test':
      return { kind: 'test', test: parsePublicTest(payload, path) };
    case 'shrink_step':
      return { kind: 'shrink_step', step: parseShrinkStep(payload, path) };
    case 'ghost':
      return { kind: 'ghost', ghost: parseGhost(payload) };
    case 'bench_point':
      return { kind: 'bench_point', point: parseBenchPoint(payload, path) };
    case 'complexity':
      return { kind: 'complexity', fact: parseComplexityFact(payload, path) };
    case 'familiar':
      return {
        kind: 'familiar',
        familiar: parseMeasuredFamiliar(payload, path)
      };
    case 'done':
      return parseDone(payload, path);
    case 'error':
      return {
        kind: 'error',
        code: readString(payload, 'code', path, 128),
        message: readString(payload, 'message', path, DISPLAY_CAP_BYTES)
      };
  }
}

/**
 * Parses one SSE message. `eventName` is the SSE `event:` field and `rawData` is its JSON `data:`.
 * Throws ContractError for unknown names, bad JSON, bad sequence numbers and bad payloads.
 */
export function parseEventEnvelope(
  eventName: string,
  rawData: string
): RunEventEnvelope {
  const path = 'event';
  if (!(RUN_EVENT_NAMES as readonly string[]).includes(eventName)) {
    throw new ContractError(path, 'unknown event name');
  }
  let decoded: unknown;
  try {
    decoded = JSON.parse(rawData);
  } catch {
    throw new ContractError(`${path}.data`, 'not JSON');
  }
  const record = asRecord(decoded, `${path}.data`);
  const payload = asRecord(record.payload, `${path}.payload`);
  const event = eventName as RunEventName;
  return {
    seq: readInt(record, 'seq', path, 1, MAX_SEQ),
    attempt: readInt(record, 'attempt', path, 1, 1000),
    event,
    payload: parsePayload(event, payload, `${path}.payload`)
  };
}
