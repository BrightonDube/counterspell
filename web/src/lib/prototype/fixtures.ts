// Fixture data for the prototype route only. Nothing here is a real problem, run or judge result.
// Production routes must never import this module. Fixture ids carry a prefix that no server id uses.
import type { ProblemDetail } from '../api/types';

/** The banner every fixture screen shows. It must stay visible while fixture data is on screen. */
export const PROTOTYPE_BANNER = 'Prototype fixture — not a real run';

const FIXTURE_RUN_PREFIX = 'fixture-run-';

/** True only for ids this module creates. Real run ids are UUIDs and never start with this prefix. */
export function isFixtureRunId(id: string): boolean {
  return id.startsWith(FIXTURE_RUN_PREFIX);
}

/** A design-only problem used to lay out the workspace before the catalogue API is available. */
export const FIXTURE_PROBLEM: ProblemDetail = {
  slug: 'fixture-two-sum',
  title: 'Fixture: Two Sum',
  track: { slug: 'fixture-arrays', name: 'Fixture track' },
  concepts: [{ slug: 'fixture-hash-maps', name: 'Fixture concept' }],
  difficulty: 'intro',
  statementMarkdown:
    'Return the indexes of two numbers that add up to the target. Layout fixture only.',
  constraints: ['Fixture constraint: two items at least.'],
  signature: {
    functionName: 'two_sum',
    parameters: [{ name: 'nums', type: 'list<int>' }],
    returnType: 'list<int>'
  },
  samples: [],
  starters: { python: '# fixture starter\ndef two_sum(nums):\n    pass\n' },
  enabledLanguages: ['python']
};
