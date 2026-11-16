// View models for `GET /api/dashboard`. The server JSON is snake_case. The parser checks every field and maps it
// to the camelCase shape the components read. A malformed response throws a ContractError, so the page can show
// its error state instead of guessing values.
import {
  asRecord,
  ContractError,
  readArray,
  readBool,
  readEnum,
  readInt,
  readOptionalInt,
  readString,
  type RawRecord
} from '../contract';

/** Catalogue difficulty levels, the same set the server's CHECK constraint allows. */
export const DIFFICULTIES = ['easy', 'medium', 'hard'] as const;
export type Difficulty = (typeof DIFFICULTIES)[number];

/** Why a problem was suggested. Both values are stable server codes. */
export const SUGGESTION_REASONS = [
  'practice_weak_concept',
  'continue_track'
] as const;
export type SuggestionReason = (typeof SUGGESTION_REASONS)[number];

/** Run lifecycle values from the run-lifecycle `submissions.status` CHECK constraint. */
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

/** User-code verdicts from the run-lifecycle `submissions.verdict` CHECK constraint. */
export const VERDICTS = [
  'accepted',
  'wrong_answer',
  'runtime_error',
  'time_limit',
  'memory_limit',
  'output_limit',
  'compile_error'
] as const;

/**
 * Titles and concept names are at most 80 characters, and one character is at most four UTF-8 bytes.
 * The cap is 80 times 4, so a valid long title is never rejected.
 */
const TEXT = 320;
/** Slugs, ids and tokens are short ASCII values. */
const TOKEN = 64;
/** A UTC timestamp as the server writes it, for example `2026-11-15T08:00:00Z`. */
const TIMESTAMP = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/;
/** A UTC calendar date, for example `2026-11-15`. */
const DATE = /^\d{4}-\d{2}-\d{2}$/;
/** A run id as a canonical UUID string. */
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
/** The server caps every list. These limits match the server and reject a response that exceeds them. */
const MAX_RECENT = 10;
const MAX_SUGGESTIONS = 3;
const MAX_WEAK = 10;
const MAX_CONCEPTS = 50;
/** Counts can grow with the catalogue and the learner's history. */
const MAX_COUNT = 1_000_000;
/** The database column for a score is a 32-bit integer. */
const INT32_MIN = -2_147_483_648;
const INT32_MAX = 2_147_483_647;

export interface SolvedSummary {
  total: number;
  byDifficulty: Record<Difficulty, number>;
}

export interface StreakSummary {
  currentDays: number;
  activeToday: boolean;
  lastActivityDate: string | null;
}

export interface WeakConcept {
  slug: string;
  name: string;
  failureCount: number;
  lastFailedAt: string;
}

export interface ProblemSummary {
  slug: string;
  title: string;
  difficulty: Difficulty;
}

export interface RecentSubmission {
  id: string;
  problem: ProblemSummary;
  lang: string;
  mode: string;
  status: string;
  verdict: string | null;
  passed: number | null;
  total: number | null;
  score: number | null;
  createdAt: string;
  finishedAt: string | null;
}

export interface ConceptRef {
  slug: string;
  name: string;
}

export interface SuggestedChallenge {
  slug: string;
  title: string;
  difficulty: Difficulty;
  concepts: ConceptRef[];
  reason: SuggestionReason;
}

export interface DashboardView {
  solved: SolvedSummary;
  streak: StreakSummary;
  weakConcepts: WeakConcept[];
  recentSubmissions: RecentSubmission[];
  suggestedChallenges: SuggestedChallenge[];
}

/** Parses the `GET /api/dashboard` body. Throws ContractError when a field is missing or out of range. */
export function parseDashboard(raw: unknown): DashboardView {
  const path = 'dashboard';
  const record = asRecord(raw, path);
  const solved = asRecord(record.solved, `${path}.solved`);
  const byDifficulty = asRecord(
    solved.by_difficulty,
    `${path}.solved.by_difficulty`
  );
  const streak = parseStreak(
    asRecord(record.streak, `${path}.streak`),
    `${path}.streak`
  );
  const weakConcepts = readArray(
    record,
    'weak_concepts',
    path,
    MAX_WEAK,
    parseWeakConcept
  );
  const recentSubmissions = readArray(
    record,
    'recent_submissions',
    path,
    MAX_RECENT,
    parseRecent
  );
  const suggestedChallenges = readArray(
    record,
    'suggested_challenges',
    path,
    MAX_SUGGESTIONS,
    parseSuggestion
  );
  requireUnique(
    weakConcepts,
    (concept) => concept.slug,
    `${path}.weak_concepts`
  );
  requireUnique(
    recentSubmissions,
    (run) => run.id,
    `${path}.recent_submissions`
  );
  requireUnique(
    suggestedChallenges,
    (challenge) => challenge.slug,
    `${path}.suggested_challenges`
  );
  return {
    solved: {
      total: readInt(solved, 'total', `${path}.solved`, 0, MAX_COUNT),
      byDifficulty: {
        easy: readInt(
          byDifficulty,
          'easy',
          `${path}.solved.by_difficulty`,
          0,
          MAX_COUNT
        ),
        medium: readInt(
          byDifficulty,
          'medium',
          `${path}.solved.by_difficulty`,
          0,
          MAX_COUNT
        ),
        hard: readInt(
          byDifficulty,
          'hard',
          `${path}.solved.by_difficulty`,
          0,
          MAX_COUNT
        )
      }
    },
    streak,
    weakConcepts,
    recentSubmissions,
    suggestedChallenges
  };
}

function parseStreak(record: RawRecord, path: string): StreakSummary {
  const currentDays = readInt(record, 'current_days', path, 0, MAX_COUNT);
  const activeToday = readBool(record, 'active_today', path);
  const lastActivityDate =
    readOptionalMatch(record, 'last_activity_date', path, DATE) ?? null;
  // A streak of days needs a last activity date, and "active today" means the streak is not empty.
  if (currentDays > 0 && lastActivityDate === null) {
    throw new ContractError(
      path,
      'a streak of days needs a last activity date'
    );
  }
  if (activeToday && currentDays === 0) {
    throw new ContractError(
      path,
      'an active learner has at least one day in the streak'
    );
  }
  return { currentDays, activeToday, lastActivityDate };
}

function parseWeakConcept(value: unknown, path: string): WeakConcept {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    name: readString(record, 'name', path, TEXT),
    failureCount: readInt(record, 'failure_count', path, 0, MAX_COUNT),
    lastFailedAt: readMatching(record, 'last_failed_at', path, TIMESTAMP)
  };
}

function parseProblem(value: unknown, path: string): ProblemSummary {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    title: readString(record, 'title', path, TEXT),
    difficulty: readEnum(record, 'difficulty', path, DIFFICULTIES)
  };
}

function parseRecent(value: unknown, path: string): RecentSubmission {
  const record = asRecord(value, path);
  return {
    id: readMatching(record, 'id', path, UUID),
    problem: parseProblem(record.problem, `${path}.problem`),
    lang: readString(record, 'lang', path, TOKEN),
    mode: readString(record, 'mode', path, TOKEN),
    status: readEnum(record, 'status', path, RUN_STATUSES),
    verdict: readOptionalEnum(record, 'verdict', path, VERDICTS),
    passed: readOptionalInt(record, 'passed', path, 0, MAX_COUNT) ?? null,
    total: readOptionalInt(record, 'total', path, 0, MAX_COUNT) ?? null,
    score: readOptionalInt(record, 'score', path, INT32_MIN, INT32_MAX) ?? null,
    createdAt: readMatching(record, 'created_at', path, TIMESTAMP),
    finishedAt:
      readOptionalMatch(record, 'finished_at', path, TIMESTAMP) ?? null
  };
}

function parseConcept(value: unknown, path: string): ConceptRef {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    name: readString(record, 'name', path, TEXT)
  };
}

function parseSuggestion(value: unknown, path: string): SuggestedChallenge {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    title: readString(record, 'title', path, TEXT),
    difficulty: readEnum(record, 'difficulty', path, DIFFICULTIES),
    concepts: readArray(record, 'concepts', path, MAX_CONCEPTS, parseConcept),
    reason: readEnum(record, 'reason', path, SUGGESTION_REASONS)
  };
}

/** Reads a required string that must match `pattern`. */
function readMatching(
  record: RawRecord,
  key: string,
  path: string,
  pattern: RegExp
): string {
  const value = readString(record, key, path, TOKEN);
  if (!pattern.test(value)) {
    throw new ContractError(`${path}.${key}`, 'has the wrong format');
  }
  return value;
}

/** Reads an optional string that must match `pattern` when it is present. */
function readOptionalMatch(
  record: RawRecord,
  key: string,
  path: string,
  pattern: RegExp
): string | undefined {
  if (record[key] === undefined || record[key] === null) return undefined;
  return readMatching(record, key, path, pattern);
}

/** Reads an optional string that must be one of the allowed values when it is present. */
function readOptionalEnum<T extends string>(
  record: RawRecord,
  key: string,
  path: string,
  allowed: readonly T[]
): T | null {
  if (record[key] === undefined || record[key] === null) return null;
  return readEnum(record, key, path, allowed);
}

/** Throws when two entries share the same key. Duplicate keys would break the page's rendering. */
function requireUnique<T>(
  items: readonly T[],
  keyOf: (item: T) => string,
  path: string
): void {
  const seen = new Set<string>();
  for (const item of items) {
    const key = keyOf(item);
    if (seen.has(key)) throw new ContractError(path, `repeats ${key}`);
    seen.add(key);
  }
}
