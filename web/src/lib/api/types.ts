// Browser view models for the catalogue, language and error contracts. Server JSON is parsed here
// and mapped to camelCase. Components only see these shapes.
import {
  asRecord,
  asString,
  readArray,
  readBool,
  readEnum,
  readInt,
  readJson,
  readOptionalInt,
  readString,
  type Json
} from '../contract';

/** Problem difficulty levels the catalogue documents. */
export const DIFFICULTIES = ['intro', 'standard', 'hard'] as const;
export type Difficulty = (typeof DIFFICULTIES)[number];

export interface Named {
  slug: string;
  name: string;
}

export interface Parameter {
  name: string;
  type: string;
}

export interface ProblemDetail {
  slug: string;
  title: string;
  track: Named;
  concepts: Named[];
  difficulty: Difficulty;
  statementMarkdown: string;
  constraints: string[];
  signature: {
    functionName: string;
    parameters: Parameter[];
    returnType: string;
  };
  samples: { input: Json; expected: Json }[];
  starters: Record<string, string>;
  enabledLanguages: string[];
}

export interface LanguageSummary {
  id: string;
  displayName: string;
  enabled: boolean;
  fileName: string;
  maxSourceBytes: number;
  supportsRun: boolean;
  supportsSubmit: boolean;
}

/** The stable error body every Axum error returns. The message is safe to show to a person. */
export interface ApiError {
  code: string;
  message: string;
  retryAfterSeconds?: number;
}

const MAX_LIST = 200;
const MAX_SOURCE_BYTES = 1024 * 1024;

function parseNamed(value: unknown, path: string): Named {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, 256),
    name: readString(record, 'name', path, 256)
  };
}

function parseParameter(value: unknown, path: string): Parameter {
  const record = asRecord(value, path);
  return {
    name: readString(record, 'name', path, 256),
    type: readString(record, 'type', path, 256)
  };
}

function parseSample(
  value: unknown,
  path: string
): { input: Json; expected: Json } {
  const record = asRecord(value, path);
  return {
    input: readJson(record, 'input', path),
    expected: readJson(record, 'expected', path)
  };
}

/** Parses `GET /api/problems/{slug}`. Missing required fields throw; nothing is filled in. */
export function parseProblemDetail(raw: unknown): ProblemDetail {
  const path = 'problem';
  const record = asRecord(raw, path);
  const signature = asRecord(record.signature, `${path}.signature`);
  const starters = asRecord(record.starters, `${path}.starters`);
  const starterMap: Record<string, string> = {};
  for (const [language, code] of Object.entries(starters)) {
    starterMap[language] = asString(
      code,
      `${path}.starters.${language}`,
      MAX_SOURCE_BYTES
    );
  }
  return {
    slug: readString(record, 'slug', path, 256),
    title: readString(record, 'title', path, 256),
    track: parseNamed(record.track, `${path}.track`),
    concepts: readArray(record, 'concepts', path, MAX_LIST, parseNamed),
    difficulty: readEnum(record, 'difficulty', path, DIFFICULTIES),
    statementMarkdown: readString(record, 'statement_markdown', path),
    constraints: readArray(
      record,
      'constraints',
      path,
      MAX_LIST,
      (value, itemPath) => asString(value, itemPath)
    ),
    signature: {
      functionName: readString(
        signature,
        'function_name',
        `${path}.signature`,
        256
      ),
      parameters: readArray(
        signature,
        'parameters',
        `${path}.signature`,
        MAX_LIST,
        parseParameter
      ),
      returnType: readString(signature, 'return_type', `${path}.signature`, 256)
    },
    samples: readArray(record, 'samples', path, MAX_LIST, parseSample),
    starters: starterMap,
    enabledLanguages: readArray(
      record,
      'enabled_languages',
      path,
      MAX_LIST,
      (value, itemPath) => asString(value, itemPath, 64)
    )
  };
}

/** Parses one entry of `GET /api/languages`. */
export function parseLanguageSummary(raw: unknown): LanguageSummary {
  const path = 'language';
  const record = asRecord(raw, path);
  return {
    id: readString(record, 'id', path, 64),
    displayName: readString(record, 'display_name', path, 128),
    enabled: readBool(record, 'enabled', path),
    fileName: readString(record, 'file_name', path, 128),
    maxSourceBytes: readInt(
      record,
      'max_source_bytes',
      path,
      1,
      MAX_SOURCE_BYTES
    ),
    supportsRun: readBool(record, 'supports_run', path),
    supportsSubmit: readBool(record, 'supports_submit', path)
  };
}

/** Parses the stable error body. Non-object bodies are refused rather than shown. */
export function parseApiError(raw: unknown): ApiError {
  const path = 'error';
  const record = asRecord(raw, path);
  const error: ApiError = {
    code: readString(record, 'code', path, 128),
    message: readString(record, 'message', path, 1024)
  };
  const retry = readOptionalInt(record, 'retry_after_seconds', path, 0, 86_400);
  if (retry !== undefined) error.retryAfterSeconds = retry;
  return error;
}
