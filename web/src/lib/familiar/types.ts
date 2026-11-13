// Browser view models for measured Familiar data. The mood may come only from measured server facts.
// Any other layer, such as an AI speech bubble, is refused here so it can never set the mood.
import {
  asRecord,
  ContractError,
  readEnum,
  readInt,
  readString
} from '../contract';

/** The five measured moods, from furious (1) to laughing (5). */
export type Mood = 1 | 2 | 3 | 4 | 5;

export const CONFIDENCE_LEVELS = ['low', 'med', 'high'] as const;
export type Confidence = (typeof CONFIDENCE_LEVELS)[number];

export interface MeasuredFamiliar {
  mood: Mood;
  reason: string;
  layer: 'measured';
}

export interface ComplexityFact {
  measuredClass: string;
  confidence: Confidence;
  reference: string;
  ratioVsBaseline: number | null;
}

export interface BenchPoint {
  n: number;
  medianNs: number;
}

const MAX_SIZE = 2 ** 40;

/** Parses a `familiar` event. Only the `measured` layer is accepted. */
export function parseMeasuredFamiliar(
  raw: unknown,
  path = 'familiar'
): MeasuredFamiliar {
  const record = asRecord(raw, path);
  const mood = readInt(record, 'mood', path, 1, 5) as Mood;
  const reason = readString(record, 'reason', path, 512);
  const layer = readEnum(record, 'layer', path, ['measured'] as const);
  return { mood, reason, layer };
}

/** Parses a `complexity` event. The ratio is null when no same-language baseline exists. */
export function parseComplexityFact(
  raw: unknown,
  path = 'complexity'
): ComplexityFact {
  const record = asRecord(raw, path);
  const ratio = record.ratio_vs_baseline;
  let ratioVsBaseline: number | null = null;
  if (ratio !== null && ratio !== undefined) {
    if (typeof ratio !== 'number' || !Number.isFinite(ratio) || ratio < 0) {
      throw new ContractError(
        `${path}.ratio_vs_baseline`,
        'expected a non-negative number or null'
      );
    }
    ratioVsBaseline = ratio;
  }
  return {
    measuredClass: readString(record, 'class', path, 64),
    confidence: readEnum(record, 'confidence', path, CONFIDENCE_LEVELS),
    reference: readString(record, 'reference', path, 64),
    ratioVsBaseline
  };
}

/** Parses one `bench_point` event: a size and its median time in nanoseconds. */
export function parseBenchPoint(
  raw: unknown,
  path = 'bench_point'
): BenchPoint {
  const record = asRecord(raw, path);
  return {
    n: readInt(record, 'n', path, 1, MAX_SIZE),
    medianNs: readInt(record, 'median_ns', path, 0, Number.MAX_SAFE_INTEGER)
  };
}
