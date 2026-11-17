import { describe, expect, it } from 'vitest';
import {
  COOLDOWN_MS,
  planEffect,
  showsBadge,
  type Effect,
  type PlanInput
} from './plan';

const NOW = 100_000;

/** A live, quiet, uncooled input for a burst. Each test changes one field. */
function base(overrides: Partial<PlanInput> = {}): PlanInput {
  return {
    effect: 'crack_burst',
    live: true,
    reducedMotion: false,
    muted: false,
    hidden: false,
    nowMs: NOW,
    lastEffectAtMs: null,
    ...overrides
  };
}

describe('planEffect', () => {
  it('plays a live crack burst that is not muted, hidden, reduced or cooled down', () => {
    expect(planEffect(base())).toEqual({ play: true });
  });

  it('plays a laugh burst the same way', () => {
    expect(planEffect(base({ effect: 'laugh_burst' }))).toEqual({ play: true });
  });

  it('skips an effect of none, and gives that as the reason', () => {
    expect(planEffect(base({ effect: 'none' }))).toEqual({
      play: false,
      why: 'none'
    });
  });

  it('skips a result that is not live, such as one loaded from history', () => {
    expect(planEffect(base({ live: false }))).toEqual({
      play: false,
      why: 'not_live'
    });
  });

  it('skips under reduced motion', () => {
    expect(planEffect(base({ reducedMotion: true }))).toEqual({
      play: false,
      why: 'reduced_motion'
    });
  });

  it('skips when muted', () => {
    expect(planEffect(base({ muted: true }))).toEqual({
      play: false,
      why: 'muted'
    });
  });

  it('skips when the Familiar is hidden', () => {
    expect(planEffect(base({ hidden: true }))).toEqual({
      play: false,
      why: 'hidden'
    });
  });

  it('skips within two seconds of the previous burst', () => {
    expect(planEffect(base({ lastEffectAtMs: NOW - 1500 }))).toEqual({
      play: false,
      why: 'cooldown'
    });
  });

  it('plays again exactly when the cooldown has passed', () => {
    expect(COOLDOWN_MS).toBe(2000);
    expect(planEffect(base({ lastEffectAtMs: NOW - COOLDOWN_MS }))).toEqual({
      play: true
    });
  });

  it('gives the first failing reason when several checks fail', () => {
    expect(planEffect(base({ live: false, muted: true }))).toEqual({
      play: false,
      why: 'not_live'
    });
    expect(
      planEffect(base({ reducedMotion: true, muted: true, hidden: true }))
    ).toEqual({
      play: false,
      why: 'reduced_motion'
    });
  });

  it('plays only when every condition holds, over every combination', () => {
    const effects: Effect[] = ['none', 'crack_burst', 'laugh_burst'];
    const flags = [false, true];
    const cooled = [false, true];
    for (const effect of effects)
      for (const live of flags)
        for (const reducedMotion of flags)
          for (const muted of flags)
            for (const hidden of flags)
              for (const onCooldown of cooled) {
                const input = base({
                  effect,
                  live,
                  reducedMotion,
                  muted,
                  hidden,
                  lastEffectAtMs: onCooldown ? NOW - 500 : null
                });
                const plan = planEffect(input);
                const allow =
                  effect !== 'none' &&
                  live &&
                  !reducedMotion &&
                  !muted &&
                  !hidden &&
                  !onCooldown;
                expect(plan.play).toBe(allow);
              }
  });
});

describe('showsBadge', () => {
  it('shows the badge text for both bursts and for no effect', () => {
    expect(showsBadge('crack_burst')).toBe(true);
    expect(showsBadge('laugh_burst')).toBe(true);
    expect(showsBadge('none')).toBe(false);
  });
});
