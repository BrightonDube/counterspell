// Component tests for the Familiar panel (ide-workspace T19). Each test names the break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import Familiar from './Familiar.svelte';
import {
  applyComplexity,
  applyMeasuredFamiliar,
  initialFamiliarState
} from './familiar-state';

afterEach(() => {
  cleanup();
});

const angryState = applyComplexity(
  applyMeasuredFamiliar(initialFamiliarState(), {
    mood: 2,
    reason: 'measured',
    layer: 'measured'
  }),
  {
    measuredClass: 'n^2',
    confidence: 'med',
    reference: 'n',
    ratioVsBaseline: 1.1
  }
);

describe('Familiar panel', () => {
  // Breaks if the measured text is not announced politely, so screen-reader users miss the result.
  it('announces the measured result in a polite live region', () => {
    render(Familiar, { props: { measured: angryState } });
    const text = screen.getByText(
      'Familiar: angry. Looks n^2 (medium confidence). Reference n.'
    );
    expect(text.closest('[aria-live="polite"]')).not.toBeNull();
  });

  // Breaks if muting does not persist for the session, so the mascot keeps animating after the learner asked it to stop.
  it('turns mute on and off through its button', async () => {
    render(Familiar, { props: { measured: angryState } });
    await fireEvent.click(screen.getByRole('button', { name: 'Mute' }));
    expect(screen.getByRole('button', { name: 'Unmute' })).toBeTruthy();
  });

  // Breaks if hiding the figure also removes the measured text that the spec keeps for screen readers.
  it('keeps the measured text after the learner hides the figure', async () => {
    render(Familiar, { props: { measured: angryState } });
    await fireEvent.click(screen.getByRole('button', { name: 'Hide' }));
    expect(screen.queryByText('Angry')).toBeNull();
    expect(
      screen.getByText(
        'Familiar: angry. Looks n^2 (medium confidence). Reference n.'
      )
    ).toBeTruthy();
  });

  // Breaks if the explanation drops the limits the spec requires the learner to see.
  it('explains the measurement and its limits in How does it judge me', () => {
    const { container } = render(Familiar, { props: { measured: angryState } });
    expect(screen.getByText('How does it judge me?')).toBeTruthy();
    const text = container.textContent ?? '';
    expect(text).toContain('medians');
    expect(text).toContain('same-language baseline');
    expect(text).toContain('no meaningful growth curve');
  });
});
