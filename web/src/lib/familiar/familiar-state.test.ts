// Tests for the measured-only Familiar state (ide-workspace T19). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import {
  applyComplexity,
  applyMeasuredFamiliar,
  applyTerminal,
  familiarView,
  initialFamiliarState,
  type FamiliarPrefs
} from './familiar-state';
import type { ComplexityFact, MeasuredFamiliar, Mood } from './types';

const noPrefs: FamiliarPrefs = { muted: false, hidden: false };

function fact(overrides: Partial<ComplexityFact> = {}): ComplexityFact {
  return {
    measuredClass: 'n^2',
    confidence: 'med',
    reference: 'n',
    ratioVsBaseline: 1.1,
    ...overrides
  };
}

function mood(value: Mood): MeasuredFamiliar {
  return { mood: value, reason: 'measured', layer: 'measured' };
}

describe('measured-only mood', () => {
  // Breaks if a complexity fact alone sets the mood, since only a measured familiar event may set it.
  it('does not set a mood from a complexity fact alone', () => {
    const state = applyComplexity(initialFamiliarState(), fact());
    expect(familiarView(state, noPrefs, false).mood).toBeNull();
  });

  // Breaks if a measured mood is lost or relabelled on the way to the view.
  it('shows the measured mood with its name', () => {
    const state = applyMeasuredFamiliar(initialFamiliarState(), mood(4));
    expect(familiarView(state, noPrefs, false)).toMatchObject({
      mood: 4,
      moodLabel: 'Pleased'
    });
  });

  // Breaks if the top state is reworded as "optimal", which the product forbids.
  it('never uses the word optimal for any mood', () => {
    for (const value of [1, 2, 3, 4, 5] as const) {
      const state = applyComplexity(
        applyMeasuredFamiliar(initialFamiliarState(), mood(value)),
        fact({ measuredClass: 'n', reference: 'n' })
      );
      const view = familiarView(state, noPrefs, false);
      expect(`${view.moodLabel} ${view.liveText}`.toLowerCase()).not.toContain(
        'optimal'
      );
    }
  });

  // Breaks if the top mood is not labelled with the measured-condition wording the spec requires.
  it('labels the top mood Matches reference complexity', () => {
    const state = applyMeasuredFamiliar(initialFamiliarState(), mood(5));
    expect(familiarView(state, noPrefs, false).moodLabel).toBe(
      'Matches reference complexity'
    );
  });
});

describe('observational text', () => {
  // Breaks if the live text states a class that was not measured, or drops the confidence.
  it('describes the measured class with its confidence and reference', () => {
    let state = applyMeasuredFamiliar(initialFamiliarState(), mood(2));
    state = applyComplexity(state, fact());
    expect(familiarView(state, noPrefs, false).liveText).toBe(
      'Familiar: angry. Looks n^2 (medium confidence). Reference n.'
    );
  });

  // Breaks if a low-confidence reading is worded as a firm verdict.
  it('calls a low-confidence reading rough', () => {
    let state = applyMeasuredFamiliar(initialFamiliarState(), mood(4));
    state = applyComplexity(
      state,
      fact({ confidence: 'low', measuredClass: 'n log n', reference: 'n' })
    );
    expect(familiarView(state, noPrefs, false).liveText).toBe(
      'Familiar: pleased. Looks n log n (low confidence, a rough reading). Reference n.'
    );
  });

  // Breaks if a problem without a bench is shown as if timing were measured.
  it('says only correctness was measured when the problem has no timing check', () => {
    const state = applyTerminal(initialFamiliarState(), {
      verdict: 'accepted',
      hasBench: false
    });
    expect(familiarView(state, noPrefs, false).liveText).toBe(
      'Correctness only. This problem has no timing check.'
    );
  });

  // Breaks if a failed run reports a mood the server never sent.
  it('reports a failed run without inventing a mood', () => {
    const state = applyTerminal(initialFamiliarState(), {
      verdict: 'wrong_answer',
      hasBench: true
    });
    const view = familiarView(state, noPrefs, false);
    expect(view.mood).toBeNull();
    expect(view.liveText).toBe('Did not pass: wrong_answer.');
  });
});

describe('preferences and motion', () => {
  // Breaks if reduced motion keeps the animation, so motion-sensitive users see effects they turned off.
  it('turns animation off under reduced motion and shows a static badge', () => {
    const state = applyMeasuredFamiliar(initialFamiliarState(), mood(5));
    const view = familiarView(state, noPrefs, true);
    expect(view.animate).toBe(false);
    expect(view.badge).toBe('Matches reference!');
  });

  // Breaks if muting also removes the text equivalent that screen readers depend on.
  it('keeps the text equivalent when the Familiar is muted', () => {
    const state = applyMeasuredFamiliar(initialFamiliarState(), mood(3));
    const view = familiarView(state, { muted: true, hidden: false }, false);
    expect(view.animate).toBe(false);
    expect(view.liveText).toBe('Familiar: uneasy.');
  });

  // Breaks if hiding the Familiar removes the text equivalent, which the spec keeps available.
  it('hides the figure but keeps the text equivalent when hidden', () => {
    const state = applyMeasuredFamiliar(initialFamiliarState(), mood(3));
    const view = familiarView(state, { muted: false, hidden: true }, false);
    expect(view.visible).toBe(false);
    expect(view.liveText).toBe('Familiar: uneasy.');
  });
});
