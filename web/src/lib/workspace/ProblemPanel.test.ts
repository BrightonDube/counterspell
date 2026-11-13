// Component tests for the problem statement panel (ide-workspace T06). Each test names the break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import type { ProblemDetail } from '../api/types';
import ProblemPanel from './ProblemPanel.svelte';

afterEach(() => {
  cleanup();
});

function problem(overrides: Partial<ProblemDetail> = {}): ProblemDetail {
  return {
    slug: 'two-sum',
    title: 'Two Sum',
    track: { slug: 'arrays', name: 'Crypt of Arrays' },
    concepts: [{ slug: 'hash-maps', name: 'Hash maps' }],
    difficulty: 'intro',
    statementMarkdown: 'Return the indices of `two` numbers.',
    constraints: ['2 <= nums.length <= 10'],
    signature: {
      functionName: 'two_sum',
      parameters: [
        { name: 'nums', type: 'list<int>' },
        { name: 'target', type: 'int' }
      ],
      returnType: 'list<int>'
    },
    samples: [{ input: { nums: [2, 7], target: 9 }, expected: [0, 1] }],
    starters: {},
    enabledLanguages: ['python'],
    ...overrides
  };
}

describe('ProblemPanel', () => {
  // Breaks if the title, track, difficulty or concept tags are dropped from the header.
  it('shows the title, track, difficulty and concept tags', () => {
    render(ProblemPanel, { props: { problem: problem() } });
    expect(
      screen.getByRole('heading', { level: 2, name: 'Two Sum' })
    ).toBeTruthy();
    expect(screen.getByText('Crypt of Arrays')).toBeTruthy();
    expect(screen.getByText('intro')).toBeTruthy();
    const concepts = screen.getByRole('list', { name: 'Concepts' });
    expect(concepts.textContent).toContain('Hash maps');
  });

  // Breaks if the signature is rendered with the wrong parameter types or order.
  it('shows the function signature as text', () => {
    render(ProblemPanel, { props: { problem: problem() } });
    expect(
      screen.getByText('two_sum(nums: list<int>, target: int) -> list<int>')
    ).toBeTruthy();
  });

  // Breaks if a statement with raw markup is injected into the DOM instead of shown as text.
  it('renders hostile statement markup as text, not elements', () => {
    const { container } = render(ProblemPanel, {
      props: {
        problem: problem({
          statementMarkdown: '<img src=x onerror="alert(1)">'
        })
      }
    });
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toContain('<img src=x onerror="alert(1)">');
  });

  // Breaks if a sample value containing a script tag is rendered as markup.
  it('renders sample values as text', () => {
    const { container } = render(ProblemPanel, {
      props: {
        problem: problem({
          samples: [{ input: { nums: ['<script>x</script>'] }, expected: 'ok' }]
        })
      }
    });
    expect(container.querySelector('script')).toBeNull();
    expect(container.textContent).toContain('<script>x</script>');
  });

  // Breaks if every constraint is not listed, so a learner misses a limit that the judge enforces.
  it('lists each constraint', () => {
    render(ProblemPanel, {
      props: {
        problem: problem({
          constraints: ['1 <= n <= 10', 'values fit in int32']
        })
      }
    });
    const constraints = screen.getByRole('list', { name: 'Constraints' });
    expect(constraints.querySelectorAll('li')).toHaveLength(2);
    expect(constraints.textContent).toContain('values fit in int32');
  });

  // Breaks if a problem with no concept tags still shows an empty concept list.
  it('omits the concept list when the problem has no concepts', () => {
    render(ProblemPanel, { props: { problem: problem({ concepts: [] }) } });
    expect(screen.queryByRole('list', { name: 'Concepts' })).toBeNull();
  });

  // Breaks if the statement cannot be collapsed from the keyboard-reachable button, or stays hidden.
  it('collapses and expands the statement from its toggle button', async () => {
    const { container } = render(ProblemPanel, {
      props: { problem: problem() }
    });
    const toggle = screen.getByRole('button', { name: 'Statement' });
    const body = container.querySelector('#statement-body') as HTMLElement;

    expect(toggle.getAttribute('aria-expanded')).toBe('true');
    expect(body.hidden).toBe(false);

    await fireEvent.click(toggle);
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    expect(body.hidden).toBe(true);

    await fireEvent.click(toggle);
    expect(toggle.getAttribute('aria-expanded')).toBe('true');
    expect(body.hidden).toBe(false);
  });
});
