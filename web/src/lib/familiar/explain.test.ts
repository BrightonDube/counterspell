// Tests for the "How does it judge me?" view-model (efficiency-familiar T34). The input is the
// `complexity` section of the run read model, as the server sends it.
import { describe, expect, it } from 'vitest';
import { explain, type ComplexitySection } from './explain';

const FORBIDDEN = [
  'optimal',
  'optimum',
  'best possible',
  'fastest possible',
  'cannot be improved'
];

/** A measured section with a clear quadratic fit and a same-language baseline ratio. */
const quadratic: ComplexitySection = {
  class: 'n^2',
  confidence: 'medium',
  reference: 'n',
  plausible: ['n^2'],
  points: [
    { n: 256, median_ns: 41000 },
    { n: 512, median_ns: 162000 }
  ],
  slope: 1.96,
  slope_se: 0.08,
  residuals: { n: 0.4, 'n^2': 0.03, 'n log n': 0.2 },
  ratio_vs_baseline: 1.1,
  fit_version: 'fit-1'
};

/** Every string the popover can show, for the wording checks. */
function allText(view: ReturnType<typeof explain>): string[] {
  const text: string[] = [...view.limits, view.summary];
  if (view.measured) {
    text.push(
      view.slopeText,
      view.confidenceText,
      view.referenceText,
      ...view.plausibleText
    );
    text.push(...view.residualRows.map((row) => row.label));
    if (view.ratioText !== null) text.push(view.ratioText);
  }
  return text;
}

describe('explain (How does it judge me?)', () => {
  // Breaks if a measured result shows no observed data, so the learner cannot check the class against the points.
  it('shows the stored points and the reference for a measured result', () => {
    const view = explain(quadratic);
    expect(view.measured).toBe(true);
    expect(view.points).toEqual([
      { n: 256, medianNs: 41000 },
      { n: 512, medianNs: 162000 }
    ]);
    expect(view.referenceText).toBe('The reference is linear.');
  });

  // Breaks if the slope is shown without its error, which would hide how sure the fit is.
  it('shows the slope with its standard error', () => {
    expect(explain(quadratic).slopeText).toBe(
      'Slope 1.96 ± 0.08 on a log-log scale.'
    );
  });

  // Breaks if medium confidence is worded as a firm verdict, or low confidence as only a rough guess.
  it('words medium and low confidence differently', () => {
    expect(explain(quadratic).confidenceText).toBe('Medium confidence.');
    const unsure = explain({ ...quadratic, class: null, confidence: 'low' });
    expect(unsure.confidenceText).toBe(
      'Unsure: the data does not single out one class.'
    );
  });

  // Breaks if an unsure fit hides the classes the data allows, which is the honest part of a low result.
  it('lists the plausible classes when the result is unsure', () => {
    const view = explain({
      ...quadratic,
      class: null,
      confidence: 'low',
      plausible: ['n', 'n log n']
    });
    expect(view.plausibleText).toEqual(['Could be linear', 'Could be n log n']);
  });

  // Breaks if the residuals are shown in an order that hides the best model, so the learner misreads the fit.
  it('sorts the per-model residuals from the best fit to the worst', () => {
    const view = explain(quadratic);
    expect(view.residualRows.map((row) => row.model)).toEqual([
      'n^2',
      'n log n',
      'n'
    ]);
    expect(view.residualRows[0].label).toBe('quadratic: residual 0.03');
  });

  // Breaks if a baseline ratio is invented when no same-language baseline exists.
  it('shows the baseline ratio only when one was measured', () => {
    expect(explain(quadratic).ratioText).toBe(
      '1.10 times the same-language baseline.'
    );
    expect(
      explain({ ...quadratic, ratio_vs_baseline: null }).ratioText
    ).toBeNull();
  });

  // Breaks if a missing section shows made-up numbers instead of saying nothing was measured.
  it('says plainly when nothing was measured', () => {
    const view = explain(null);
    expect(view.measured).toBe(false);
    expect(view.summary).toBe(
      'This run was not timed, so there is no growth reading.'
    );
    expect(view.points).toEqual([]);
    expect(view.residualRows).toEqual([]);
  });

  // Breaks if the honest limits are dropped from the popover, which the spec requires the learner to see.
  it('always includes the honest limits from the design', () => {
    const view = explain(quadratic);
    expect(view.limits.length).toBeGreaterThanOrEqual(4);
    expect(view.limits.join(' ')).toContain('medians');
    expect(view.limits.join(' ')).toContain('shared');
  });

  // Breaks if the popover uses the word the product forbids for the top mood, anywhere in its text.
  it('never uses a forbidden word in any text it produces', () => {
    for (const section of [
      quadratic,
      null,
      { ...quadratic, class: null, confidence: 'low' as const }
    ]) {
      for (const text of allText(
        explain(section as ComplexitySection | null)
      )) {
        for (const word of FORBIDDEN) {
          expect(text.toLowerCase()).not.toContain(word);
        }
      }
    }
  });
});
