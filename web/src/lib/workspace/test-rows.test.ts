// Tests for the test-row view model (ide-workspace T12). The rows come from the public run read model.
import { describe, expect, it } from 'vitest';
import type { PublicTest } from '../runs/types';
import {
  OUTPUT_DISPLAY_CHARS,
  rowView,
  runOutcomeLabel,
  statusLabel
} from './test-rows';

const sampleRow: PublicTest = {
  sample: true,
  ordinal: 1,
  verdict: 'wrong_answer',
  timeMs: 7,
  input: '[1, 2]',
  expected: '3',
  got: '2'
};

const hiddenRow: PublicTest = {
  sample: false,
  ordinal: 4,
  verdict: 'time_limit',
  timeMs: 2000
};

describe('rowView', () => {
  it('shows a sample row with its input, expected and got values as plain strings', () => {
    expect(rowView(sampleRow)).toEqual({
      kind: 'sample',
      ordinal: 1,
      label: 'Wrong answer',
      icon: '✗',
      timeLabel: '7 ms',
      input: '[1, 2]',
      expected: '3',
      got: '2'
    });
  });

  // Breaks if a hidden row ever gains input, expected or got fields in its view model.
  it('shows a hidden row with ordinal, verdict and time only', () => {
    const view = rowView(hiddenRow);
    expect(view).toEqual({
      kind: 'hidden',
      ordinal: 4,
      label: 'Time limit exceeded',
      icon: '✗',
      timeLabel: '2000 ms'
    });
    expect(Object.keys(view)).not.toContain('input');
    expect(Object.keys(view)).not.toContain('expected');
    expect(Object.keys(view)).not.toContain('got');
  });

  it('gives a passing row the Passed label and a tick icon', () => {
    const view = rowView({
      sample: false,
      ordinal: 2,
      verdict: 'pass',
      timeMs: 3
    });
    expect(view.label).toBe('Passed');
    expect(view.icon).toBe('✓');
  });

  // Breaks if a very long program output fills the page or the browser's memory.
  it('truncates a long sample value and marks the cut', () => {
    const view = rowView({
      ...sampleRow,
      got: 'a'.repeat(OUTPUT_DISPLAY_CHARS + 10)
    });
    if (view.kind !== 'sample') throw new Error('expected a sample row');
    expect(view.got).toBe('a'.repeat(OUTPUT_DISPLAY_CHARS) + '… (truncated)');
  });
});

describe('labels', () => {
  it('gives each run status a distinct plain-English label', () => {
    expect(statusLabel('queued')).toBe('Queued');
    expect(statusLabel('compiling')).toBe('Compiling');
    expect(statusLabel('running')).toBe('Running');
    expect(statusLabel('ghost_hunting')).toBe('Hunting a ghost');
    expect(statusLabel('internal_error')).toBe(
      'Internal error (not a wrong answer)'
    );
  });

  // Breaks if a compile error, a timeout, a memory limit and an internal error all look the same.
  it('keeps compile, runtime, time, memory, output and internal outcomes distinct', () => {
    const labels = [
      runOutcomeLabel({
        kind: 'done',
        verdict: 'compile_error',
        passed: 0,
        total: 3,
        score: 0
      }),
      runOutcomeLabel({
        kind: 'done',
        verdict: 'runtime_error',
        passed: 0,
        total: 3,
        score: 0
      }),
      runOutcomeLabel({
        kind: 'done',
        verdict: 'time_limit',
        passed: 0,
        total: 3,
        score: 0
      }),
      runOutcomeLabel({
        kind: 'done',
        verdict: 'memory_limit',
        passed: 0,
        total: 3,
        score: 0
      }),
      runOutcomeLabel({
        kind: 'done',
        verdict: 'output_limit',
        passed: 0,
        total: 3,
        score: 0
      }),
      runOutcomeLabel({ kind: 'error', code: 'internal_error', message: 'x' })
    ];
    expect(new Set(labels).size).toBe(labels.length);
    expect(labels[0]).toBe('Compile error');
    expect(labels[5]).toBe('Internal error (not a wrong answer)');
  });
});
