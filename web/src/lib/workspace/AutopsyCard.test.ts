// Component tests for the Bug Autopsy card (ide-workspace T16). Each test names the break it catches.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import AutopsyCard from './AutopsyCard.svelte';
import { parseGhost, type GhostView } from '../ghost/types';

afterEach(() => {
  cleanup();
});

function ghostWith(overrides: Record<string, unknown> = {}): GhostView {
  return parseGhost({
    kind: 'wrong',
    input: [0, 0],
    expected: [0, 1],
    got: [0, 0],
    explanation: 'You returned [0, 0]; expected [0, 1] for [0, 0].',
    budget_exhausted: false,
    shrink: {
      steps: [],
      total_steps: 0,
      omitted_steps: 0,
      stop: 'no_smaller_failure'
    },
    provenance: {
      seed: 42,
      generator_version: 'gen-1',
      engine_version: 'engine-1',
      problem_version: 2,
      image_digest: 'sha256:abc'
    },
    autopsy: {
      explanation: 'The inner loop reuses the same index.',
      differences: [{ path: '[1]', expected: 1, got: 0 }],
      differences_truncated: false,
      concept_tags: ['hash-maps'],
      authored_mistake: null
    },
    ...overrides
  });
}

describe('AutopsyCard', () => {
  // Breaks if the expected and actual values are not shown as text with a label, so colour alone carries meaning.
  it('shows expected and actual values with text labels', () => {
    render(AutopsyCard, { props: { ghost: ghostWith() } });
    expect(screen.getByText('Expected')).toBeTruthy();
    expect(screen.getByText('Got')).toBeTruthy();
    expect(screen.getByText('[0,1]')).toBeTruthy();
  });

  // Breaks if hostile output is parsed as markup, which would run script in the learner's browser.
  it('renders script text from the explanation as plain text', () => {
    const { container } = render(AutopsyCard, {
      props: {
        ghost: ghostWith({
          explanation: '<script>alert("x")</script> returned the wrong value.'
        })
      }
    });
    expect(container.querySelector('script')).toBeNull();
    expect(
      screen.getByText(
        /<script>alert\("x"\)<\/script> returned the wrong value\./
      )
    ).toBeTruthy();
  });

  // Breaks if a payload missing its expected value is shown with guessed values instead of an unavailable state.
  it('shows an unavailable message for an incomplete payload', () => {
    const incomplete = ghostWith();
    const { expected: _omitted, ...rest } = incomplete;
    render(AutopsyCard, {
      props: { ghost: { ...rest, expected: undefined } as GhostView }
    });
    expect(screen.getByText(/cannot compare them/i)).toBeTruthy();
    expect(screen.queryByText('[0,1]')).toBeNull();
  });

  // Breaks if the card with no ghost shows an empty result panel that looks like a finished autopsy.
  it('explains that there is no autopsy yet when no ghost exists', () => {
    render(AutopsyCard, { props: { ghost: null } });
    expect(
      screen.getByText(/a failing submission produces the bug autopsy/i)
    ).toBeTruthy();
  });

  // Breaks if an authored explanation is shown without its label, so learners cannot tell it apart from engine output.
  it('labels an authored explanation exactly', () => {
    render(AutopsyCard, {
      props: {
        ghost: ghostWith({
          autopsy: {
            explanation: 'Engine line.',
            differences: [],
            differences_truncated: false,
            concept_tags: [],
            authored_mistake: {
              known_ghost_id: 'same-index',
              text: 'You reused index i.'
            }
          }
        })
      }
    });
    expect(screen.getByText('Authored explanation')).toBeTruthy();
    expect(screen.getByText('You reused index i.')).toBeTruthy();
  });
});
