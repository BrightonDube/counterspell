// Component tests for the sample and hidden test panel (ide-workspace T12). Each test names the break it catches.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import type { PublicTest } from '../runs/types';
import TestPanel from './TestPanel.svelte';

afterEach(() => {
  cleanup();
});

const sample: PublicTest = {
  sample: true,
  ordinal: 1,
  verdict: 'wrong_answer',
  timeMs: 7,
  input: '[1, 2]',
  expected: '3',
  got: '2'
};

describe('TestPanel', () => {
  it('shows sample input, expected and got, plus the verdict and time', () => {
    const { container } = render(TestPanel, {
      props: { status: 'completed', tests: [sample] }
    });
    expect(screen.getByText('Test 1: Wrong answer, 7 ms')).toBeTruthy();
    expect(container.textContent).toContain('[1, 2]');
    expect(container.textContent).toContain('3');
    expect(container.textContent).toContain('2');
  });

  // Breaks if a hidden test's input or expected output reaches the DOM, even when the type says it can't.
  it('never renders hidden detail, even if a hidden row carries extra fields', () => {
    const leaky = {
      sample: false,
      ordinal: 2,
      verdict: 'pass',
      timeMs: 4,
      input: 'HIDDEN-CANARY-INPUT',
      expected: 'HIDDEN-CANARY-EXPECTED'
    } as unknown as PublicTest;
    const { container } = render(TestPanel, {
      props: { status: 'running', tests: [leaky] }
    });
    expect(screen.getByText('Test 2: Passed, 4 ms')).toBeTruthy();
    expect(container.textContent).not.toContain('HIDDEN-CANARY');
  });

  // Breaks if program output that looks like HTML runs in the browser.
  it('keeps program output as text, never as markup', () => {
    const { container } = render(TestPanel, {
      props: {
        status: 'completed',
        tests: [{ ...sample, got: '<script>alert(1)</script>' }]
      }
    });
    expect(container.querySelector('script')).toBeNull();
    expect(container.textContent).toContain('<script>alert(1)</script>');
  });

  // Breaks if a compile error is shown as a wrong answer, or an internal error is blamed on the learner.
  it('shows the terminal outcome in words', () => {
    render(TestPanel, {
      props: {
        status: 'completed',
        tests: [],
        terminal: {
          kind: 'done',
          verdict: 'compile_error',
          passed: 0,
          total: 3,
          score: 0
        }
      }
    });
    expect(screen.getByText('Compile error')).toBeTruthy();
  });

  // Breaks if rows from a previous attempt stay on screen after a restart.
  it('renders no rows when the attempt has none', () => {
    render(TestPanel, { props: { status: 'queued', tests: [] } });
    expect(screen.queryAllByRole('listitem')).toHaveLength(0);
    expect(screen.getByText('Queued')).toBeTruthy();
  });
});
