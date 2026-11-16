// Tests for the measured Familiar contract (ide-workspace T01, system-design §8b). Each test names the break it catches.
import { describe, expect, it } from 'vitest';
import { ContractError } from '../contract';
import {
  parseBenchPoint,
  parseComplexityFact,
  parseMeasuredFamiliar
} from './types';

describe('parseMeasuredFamiliar', () => {
  // Breaks if an AI-labelled or unknown layer can set the mood, which must come only from measurement.
  it('rejects a familiar event whose layer is not measured', () => {
    expect(() =>
      parseMeasuredFamiliar({ mood: 4, reason: 'nice', layer: 'ai' })
    ).toThrow(ContractError);
  });

  // Breaks if a mood outside the five documented states is shown as a sixth face.
  it('rejects a mood outside one to five', () => {
    expect(() =>
      parseMeasuredFamiliar({ mood: 6, reason: 'x', layer: 'measured' })
    ).toThrow(ContractError);
  });

  // Breaks if a valid measured mood loses its reason text on the way to the component.
  it('accepts a measured mood with its reason', () => {
    expect(
      parseMeasuredFamiliar({
        mood: 5,
        reason: 'Matches reference complexity',
        layer: 'measured'
      })
    ).toEqual({
      mood: 5,
      reason: 'Matches reference complexity',
      layer: 'measured'
    });
  });
});

describe('parseComplexityFact', () => {
  // Breaks if an unknown confidence level is accepted, since low confidence must never lower the mood a step too far.
  it('rejects a confidence level outside low, med and high', () => {
    expect(() =>
      parseComplexityFact({
        class: 'n^2',
        confidence: 'certain',
        reference: 'n',
        ratio_vs_baseline: 1.2
      })
    ).toThrow(ContractError);
  });

  // Breaks if a missing baseline ratio is treated as a measured ratio of zero.
  it('keeps a missing baseline ratio as null', () => {
    expect(
      parseComplexityFact({
        class: 'n',
        confidence: 'med',
        reference: 'n',
        ratio_vs_baseline: null
      })
    ).toEqual({
      measuredClass: 'n',
      confidence: 'med',
      reference: 'n',
      ratioVsBaseline: null
    });
  });
});

describe('parseBenchPoint', () => {
  // Breaks if a bench point with a negative time reaches the mini chart as a real measurement.
  it('rejects a negative median time', () => {
    expect(() => parseBenchPoint({ n: 256, median_ns: -1 })).toThrow(
      ContractError
    );
  });
});
