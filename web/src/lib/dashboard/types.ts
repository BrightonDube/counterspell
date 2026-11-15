// View models for `GET /api/dashboard`. The server JSON is snake_case. The parser checks every field and maps it
// to the camelCase shape the components read. A malformed response throws a ContractError, so the page can show
// its error state instead of guessing values.
import {
  asRecord,
  readArray,
  readBool,
  readEnum,
  readInt,
  readOptionalInt,
  readOptionalString,
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

/** A short display name or slug. */
const SHORT = 120;
/** Ids and timestamps are short ASCII values. */
const TOKEN = 64;
/** The server caps every list. These limits match the server and reject a response that exceeds them. */
const MAX_RECENT = 10;
const MAX_SUGGESTIONS = 3;
const MAX_WEAK = 200;
const MAX_CONCEPTS = 50;
/** Counts can grow with the catalogue and the learner's history. */
const MAX_COUNT = 1_000_000;

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
  const streak = asRecord(record.streak, `${path}.streak`);
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
    streak: {
      currentDays: readInt(
        streak,
        'current_days',
        `${path}.streak`,
        0,
        MAX_COUNT
      ),
      activeToday: readBool(streak, 'active_today', `${path}.streak`),
      lastActivityDate:
        readOptionalString(
          streak,
          'last_activity_date',
          `${path}.streak`,
          TOKEN
        ) ?? null
    },
    weakConcepts: readArray(
      record,
      'weak_concepts',
      path,
      MAX_WEAK,
      parseWeakConcept
    ),
    recentSubmissions: readArray(
      record,
      'recent_submissions',
      path,
      MAX_RECENT,
      parseRecent
    ),
    suggestedChallenges: readArray(
      record,
      'suggested_challenges',
      path,
      MAX_SUGGESTIONS,
      parseSuggestion
    )
  };
}

function parseWeakConcept(value: unknown, path: string): WeakConcept {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    name: readString(record, 'name', path, SHORT),
    failureCount: readInt(record, 'failure_count', path, 0, MAX_COUNT),
    lastFailedAt: readString(record, 'last_failed_at', path, TOKEN)
  };
}

function parseProblem(value: unknown, path: string): ProblemSummary {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    title: readString(record, 'title', path, SHORT),
    difficulty: readEnum(record, 'difficulty', path, DIFFICULTIES)
  };
}

function parseRecent(value: unknown, path: string): RecentSubmission {
  const record: RawRecord = asRecord(value, path);
  return {
    id: readString(record, 'id', path, TOKEN),
    problem: parseProblem(record.problem, `${path}.problem`),
    lang: readString(record, 'lang', path, TOKEN),
    mode: readString(record, 'mode', path, TOKEN),
    status: readString(record, 'status', path, TOKEN),
    verdict: readOptionalString(record, 'verdict', path, TOKEN) ?? null,
    passed: readOptionalInt(record, 'passed', path, 0, MAX_COUNT) ?? null,
    total: readOptionalInt(record, 'total', path, 0, MAX_COUNT) ?? null,
    score: readOptionalInt(record, 'score', path, 0, MAX_COUNT) ?? null,
    createdAt: readString(record, 'created_at', path, TOKEN),
    finishedAt: readOptionalString(record, 'finished_at', path, TOKEN) ?? null
  };
}

function parseConcept(value: unknown, path: string): ConceptRef {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    name: readString(record, 'name', path, SHORT)
  };
}

function parseSuggestion(value: unknown, path: string): SuggestedChallenge {
  const record = asRecord(value, path);
  return {
    slug: readString(record, 'slug', path, TOKEN),
    title: readString(record, 'title', path, SHORT),
    difficulty: readEnum(record, 'difficulty', path, DIFFICULTIES),
    concepts: readArray(record, 'concepts', path, MAX_CONCEPTS, parseConcept),
    reason: readEnum(record, 'reason', path, SUGGESTION_REASONS)
  };
}
