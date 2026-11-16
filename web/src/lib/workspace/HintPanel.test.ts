// Component tests for the hint panel's AI hint display (ide-workspace T17).
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import HintPanel from './HintPanel.svelte';
import { initialLadder } from './hint-ladder';

afterEach(() => {
  cleanup();
});

const ladder = {
  ...initialLadder(2),
  revealed: [{ rung: 1, text: 'Check the pairs you have seen.' }]
};

describe('HintPanel AI hint', () => {
  // Breaks if AI text appears without the label that tells the learner it came from an AI.
  it('labels AI text with the exact words AI hint', () => {
    render(HintPanel, {
      props: {
        ladder,
        onNext: () => {},
        aiHint: { text: 'Your inner loop repeats the first index.' }
      }
    });
    expect(screen.getByText('AI hint')).toBeTruthy();
    expect(
      screen.getByText('Your inner loop repeats the first index.')
    ).toBeTruthy();
  });

  // Breaks if the server's hint text is rendered as HTML, which would let it run markup in the page.
  it('keeps AI text inert: markup is shown as text', () => {
    const { container } = render(HintPanel, {
      props: {
        ladder,
        onNext: () => {},
        aiHint: { text: '<img src=x onerror="alert(1)">' }
      }
    });
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toContain('<img src=x onerror="alert(1)">');
  });

  // Breaks if an AI label appears when no AI text was ever returned.
  it('shows no AI section when there is no AI hint', () => {
    render(HintPanel, { props: { ladder, onNext: () => {}, aiHint: null } });
    expect(screen.queryByText('AI hint')).toBeNull();
  });

  // Breaks if the authored rung disappears when the AI hint is added.
  it('still shows the revealed authored rung in order', () => {
    render(HintPanel, { props: { ladder, onNext: () => {}, aiHint: null } });
    expect(screen.getByText('Hint 1')).toBeTruthy();
    expect(screen.getByText('Check the pairs you have seen.')).toBeTruthy();
  });
});
