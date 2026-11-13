// Tests for the problem and language contracts (ide-workspace T01). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ContractError } from '../contract';
import {
  parseApiError,
  parseLanguageSummary,
  parseProblemDetail
} from './types';

const problem = {
  slug: 'two-sum',
  title: 'Two Sum',
  track: { slug: 'arrays', name: 'Crypt of Arrays' },
  concepts: [{ slug: 'hash-maps', name: 'Hash maps' }],
  difficulty: 'intro',
  statement_markdown: 'Find two indexes.',
  constraints: ['2 <= n <= 10'],
  signature: {
    function_name: 'two_sum',
    parameters: [{ name: 'nums', type: 'list<int>' }],
    return_type: 'list<int>'
  },
  samples: [{ input: { nums: [2, 7] }, expected: [0, 1] }],
  starters: { python: 'def two_sum(nums):\n    pass\n' },
  enabled_languages: ['python', 'javascript']
};

describe('parseProblemDetail', () => {
  // Breaks if the adapter drops a documented field or renames it on the way to the view model.
  it('maps a documented problem into the camelCase view model', () => {
    const parsed = parseProblemDetail(problem);
    expect(parsed.statementMarkdown).toBe('Find two indexes.');
    expect(parsed.signature.functionName).toBe('two_sum');
    expect(parsed.samples).toEqual([
      { input: { nums: [2, 7] }, expected: [0, 1] }
    ]);
    expect(parsed.enabledLanguages).toEqual(['python', 'javascript']);
  });

  // Breaks if a missing required field is silently replaced with an empty value.
  it('rejects a problem without a title', () => {
    const { title: _omitted, ...withoutTitle } = problem;
    expect(() => parseProblemDetail(withoutTitle)).toThrow(ContractError);
  });

  // Breaks if an unknown difficulty reaches components and renders as a fabricated level.
  it('rejects a difficulty outside the documented set', () => {
    expect(() =>
      parseProblemDetail({ ...problem, difficulty: 'legendary' })
    ).toThrow(ContractError);
  });

  // Breaks if the bundle or the view model accepts an oversized catalogue text without limit.
  it('rejects a statement larger than the display cap', () => {
    const huge = 'a'.repeat(64 * 1024 + 1);
    expect(() =>
      parseProblemDetail({ ...problem, statement_markdown: huge })
    ).toThrow(ContractError);
  });
});

describe('parseLanguageSummary', () => {
  // Breaks if a language the server has disabled is shown as selectable.
  it('keeps the server enabled flag as given', () => {
    const parsed = parseLanguageSummary({
      id: 'cpp',
      display_name: 'C++',
      enabled: false,
      file_name: 'solution.cpp',
      max_source_bytes: 65536,
      supports_run: false,
      supports_submit: false
    });
    expect(parsed.enabled).toBe(false);
    expect(parsed.maxSourceBytes).toBe(65536);
  });

  // Breaks if a zero or negative byte limit lets the editor accept unlimited source.
  it('rejects a non-positive source byte limit', () => {
    expect(() =>
      parseLanguageSummary({
        id: 'python',
        display_name: 'Python',
        enabled: true,
        file_name: 'solution.py',
        max_source_bytes: 0,
        supports_run: true,
        supports_submit: true
      })
    ).toThrow(ContractError);
  });
});

describe('parseApiError', () => {
  // Breaks if the stable code or safe message is lost, or a retry hint is dropped for rate limits.
  it('reads the stable code, message and retry time', () => {
    expect(
      parseApiError({
        code: 'rate_limited',
        message: 'Try again soon.',
        retry_after_seconds: 42
      })
    ).toEqual({
      code: 'rate_limited',
      message: 'Try again soon.',
      retryAfterSeconds: 42
    });
  });

  // Breaks if an error body with a non-object shape crashes the client instead of being refused.
  it('rejects an error body that is not an object', () => {
    expect(() => parseApiError('internal')).toThrow(ContractError);
  });
});
