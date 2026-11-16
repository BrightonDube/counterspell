// Browser view model for the ghost payload (ghost-engine provenance and recorded shrink trace).
// Every value comes from the trusted judge. Missing provenance or unknown kinds are refused.
import {
  asRecord,
  asString,
  readArray,
  readBool,
  readEnum,
  readInt,
  readJson,
  readString,
  type Json,
  type RawRecord
} from '../contract';

/** The engine's failure kinds, as the public payload names them. */
export const GHOST_KINDS = ['wrong', 'crash', 'timeout'] as const;
export type GhostKind = (typeof GHOST_KINDS)[number];

const MAX_STEPS = 1000;
const MAX_SAFE = Number.MAX_SAFE_INTEGER;

export interface ShrinkStepView {
  index: number;
  input: Json;
  evaluations: number;
}

export interface ShrinkTraceView {
  steps: ShrinkStepView[];
  totalSteps: number;
  omittedSteps: number;
  stop: string;
}

export interface Provenance {
  seed: number;
  generatorVersion: string;
  engineVersion: string;
  problemVersion: number;
  imageDigest: string;
}

export interface Difference {
  path: string;
  expected?: Json;
  got?: Json;
}

export interface AuthoredMistake {
  knownGhostId: string;
  text: string;
}

export interface AutopsyView {
  explanation: string;
  differences: Difference[];
  differencesTruncated: boolean;
  conceptTags: string[];
  authoredMistake: AuthoredMistake | null;
}

export interface GhostView {
  kind: GhostKind;
  input: Json;
  expected?: Json;
  got?: Json;
  explanation: string;
  budgetExhausted: boolean;
  shrink: ShrinkTraceView;
  provenance: Provenance;
  autopsy: AutopsyView;
}

/** Reads an optional JSON value. Missing and null both give undefined. */
function optionalJson(
  source: RawRecord,
  key: string,
  path: string
): Json | undefined {
  if (source[key] === undefined || source[key] === null) return undefined;
  return readJson(source, key, path);
}

/** Parses one recorded shrink step. The input must be present; the browser never invents one. */
export function parseShrinkStep(
  raw: unknown,
  path = 'shrink_step'
): ShrinkStepView {
  const record = asRecord(raw, path);
  return {
    index: readInt(record, 'index', path, 1, MAX_STEPS),
    input: readJson(record, 'input', path),
    evaluations: readInt(record, 'evaluations', path, 0, MAX_SAFE)
  };
}

function parseTrace(raw: unknown, path: string): ShrinkTraceView {
  const record = asRecord(raw, path);
  const steps = readArray(record, 'steps', path, MAX_STEPS, (value, itemPath) =>
    parseShrinkStep(value, itemPath)
  );
  return {
    steps,
    totalSteps: readInt(record, 'total_steps', path, 0, MAX_SAFE),
    omittedSteps: readInt(record, 'omitted_steps', path, 0, MAX_SAFE),
    stop: readString(record, 'stop', path, 128)
  };
}

function parseProvenance(raw: unknown, path: string): Provenance {
  const record = asRecord(raw, path);
  return {
    seed: readInt(record, 'seed', path, 0, MAX_SAFE),
    generatorVersion: readString(record, 'generator_version', path, 128),
    engineVersion: readString(record, 'engine_version', path, 128),
    problemVersion: readInt(record, 'problem_version', path, 1, MAX_SAFE),
    imageDigest: readString(record, 'image_digest', path, 256)
  };
}

function parseDifference(raw: unknown, path: string): Difference {
  const record = asRecord(raw, path);
  const difference: Difference = {
    path: readString(record, 'path', path, 512)
  };
  const expected = optionalJson(record, 'expected', path);
  const got = optionalJson(record, 'got', path);
  if (expected !== undefined) difference.expected = expected;
  if (got !== undefined) difference.got = got;
  return difference;
}

function parseAuthored(raw: unknown, path: string): AuthoredMistake | null {
  if (raw === undefined || raw === null) return null;
  const record = asRecord(raw, path);
  return {
    knownGhostId: readString(record, 'known_ghost_id', path, 128),
    text: readString(record, 'text', path)
  };
}

function parseAutopsy(raw: unknown, path: string): AutopsyView {
  const record = asRecord(raw, path);
  return {
    explanation: readString(record, 'explanation', path),
    differences: readArray(record, 'differences', path, 100, parseDifference),
    differencesTruncated: readBool(record, 'differences_truncated', path),
    conceptTags: readArray(
      record,
      'concept_tags',
      path,
      50,
      (value, itemPath) => asString(value, itemPath, 128)
    ),
    authoredMistake: parseAuthored(
      record.authored_mistake,
      `${path}.authored_mistake`
    )
  };
}

/** Parses the ghost payload. A ghost without provenance or with an unknown kind throws. */
export function parseGhost(raw: unknown): GhostView {
  const path = 'ghost';
  const record = asRecord(raw, path);
  const ghost: GhostView = {
    kind: readEnum(record, 'kind', path, GHOST_KINDS),
    input: readJson(record, 'input', path),
    explanation: readString(record, 'explanation', path),
    budgetExhausted: readBool(record, 'budget_exhausted', path),
    shrink: parseTrace(record.shrink, `${path}.shrink`),
    provenance: parseProvenance(record.provenance, `${path}.provenance`),
    autopsy: parseAutopsy(record.autopsy, `${path}.autopsy`)
  };
  const expected = optionalJson(record, 'expected', path);
  const got = optionalJson(record, 'got', path);
  if (expected !== undefined) ghost.expected = expected;
  if (got !== undefined) ghost.got = got;
  return ghost;
}
