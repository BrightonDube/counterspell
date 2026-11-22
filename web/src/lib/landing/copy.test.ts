import { describe, expect, it } from 'vitest';
import {
  BEATS,
  BENEFITS,
  CAPTION,
  CTA,
  EXAMPLE,
  HEADLINE,
  HERO_BODY,
  INTRO_BODY,
  SUPPORT_LINE
} from './copy';

describe('landing copy', () => {
  it('uses the approved Ghost Engine headline and hero description', () => {
    expect(HEADLINE).toBe("Don't Just Fix the Code. Understand the Mistake.");
    expect(HERO_BODY).toContain('AI can give you answers');
    expect(HERO_BODY).toContain('solve problems without relying on AI');
  });

  it('lists the three beats in order: Submit, Hunt, Understand', () => {
    expect(BEATS.map((beat) => beat.heading)).toEqual([
      'Submit',
      'Hunt',
      'Understand'
    ]);
    for (const beat of BEATS) {
      expect(beat.body.length).toBeGreaterThan(0);
    }
  });

  it('describes the Hunt result as a reduced failing case', () => {
    const hunt = BEATS.find((beat) => beat.heading === 'Hunt');
    expect(hunt?.body).toContain('counterexample');
    expect(hunt?.body).toContain('simpler failing case');
  });

  it('states the three independent-thinking benefits', () => {
    expect(BENEFITS.map((benefit) => benefit.heading)).toEqual([
      'Think independently',
      'Learn from failure',
      'Build real mastery'
    ]);
  });

  it('labels the example as an illustration with the fixed values', () => {
    expect(CAPTION).toBe('Illustration, not a real run.');
    expect(EXAMPLE.input).toBe('nums = [0, 0], target = 0');
    expect(EXAMPLE.expected).toBe('[0, 1]');
    expect(EXAMPLE.got).toBe('[0, 0]');
  });

  it('has the call to action label', () => {
    expect(CTA.primary).toBe('Start Your First Challenge');
  });

  it('has a non-empty supporting line', () => {
    expect(SUPPORT_LINE.length).toBeGreaterThan(0);
  });

  it('has an intro paragraph under the headline that is not empty (T25)', () => {
    expect(INTRO_BODY.trim().length).toBeGreaterThan(0);
  });

  it('never claims a smallest possible input, a proven minimum or an optimal result (R9)', () => {
    const visible = [
      HEADLINE,
      HERO_BODY,
      SUPPORT_LINE,
      INTRO_BODY,
      CAPTION,
      ...Object.values(EXAMPLE),
      ...Object.values(CTA),
      ...BEATS.flatMap((beat) => [beat.heading, beat.body])
    ].join('\n');
    expect(visible).not.toMatch(/smallest possible|proven minimum|optimal/i);
  });
});
