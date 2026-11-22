// Tests for the console view model (costume-polish T06). Each test names the safety or
// ordering boundary that keeps public program output useful without leaking hidden detail.
import { describe, expect, it } from 'vitest';
import type { PublicTest } from '../runs/types';
import { buildConsoleEntries, OUTPUT_DISPLAY_CHARS } from './console-view';

const sample: PublicTest = {
  sample: true,
  ordinal: 2,
  verdict: 'wrong_answer',
  timeMs: 7,
  input: '[1]',
  expected: '2',
  got: '1',
  stderr: 'debug'
};

describe('buildConsoleEntries', () => {
  it('puts compile output first, then public sample output and stderr in test order', () => {
    const first: PublicTest = {
      sample: true,
      ordinal: 1,
      verdict: 'pass',
      timeMs: 2,
      input: '[]',
      expected: '0',
      got: 'first stdout',
      stderr: 'first stderr'
    };

    expect(
      buildConsoleEntries({
        tests: [sample, first],
        terminal: {
          kind: 'done',
          verdict: 'wrong_answer',
          passed: 1,
          total: 2,
          score: 50,
          compileOutput: 'compiler warning'
        }
      }).map((entry) => [entry.heading, entry.text])
    ).toEqual([
      ['Compile output', 'compiler warning'],
      ['Sample 1 output', 'first stdout'],
      ['Sample 1 stderr', 'first stderr'],
      ['Sample 2 output', '1'],
      ['Sample 2 stderr', 'debug']
    ]);
  });

  it('bounds each console block with the existing display cap', () => {
    const longOutput = 'x'.repeat(OUTPUT_DISPLAY_CHARS + 20);
    const entries = buildConsoleEntries({
      tests: [{ ...sample, got: longOutput }]
    });

    expect(entries).toHaveLength(2);
    expect(entries[0].text).toBe(
      `${'x'.repeat(OUTPUT_DISPLAY_CHARS)}… (truncated)`
    );
    expect(entries[1].text).toBe('debug');
  });

  it('returns no entries for hidden tests because they carry no public output', () => {
    expect(
      buildConsoleEntries({
        tests: [
          {
            sample: false,
            ordinal: 3,
            verdict: 'runtime_error',
            timeMs: 9
          }
        ]
      })
    ).toEqual([]);
  });
});
