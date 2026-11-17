// Effect planner: decides whether a crack or laugh burst plays now. Pure: the caller supplies the
// clock reading and the preferences. A skipped effect never stops the mood from showing.

/** The effect a measured result asks for. `none` means no burst. */
export type Effect = 'none' | 'crack_burst' | 'laugh_burst';

/** Everything the planner needs. */
export interface PlanInput {
  effect: Effect;
  /** True only for a live result on the page, not a result loaded from history. */
  live: boolean;
  reducedMotion: boolean;
  muted: boolean;
  hidden: boolean;
  nowMs: number;
  /** When the previous burst started, or null when none has played. */
  lastEffectAtMs: number | null;
}

/** Why a burst was skipped. `none` means the result asked for no burst. */
export type SkipReason =
  'none' | 'not_live' | 'reduced_motion' | 'muted' | 'hidden' | 'cooldown';

/** The planner's decision: play, or skip with the reason. */
export type EffectPlan = { play: true } | { play: false; why: SkipReason };

/** Bursts are at least this far apart, in milliseconds. */
export const COOLDOWN_MS = 2000;

/** Decides whether the burst plays now. The checks run in a fixed order, so one reason is given. */
export function planEffect(input: PlanInput): EffectPlan {
  if (input.effect === 'none') return { play: false, why: 'none' };
  if (!input.live) return { play: false, why: 'not_live' };
  if (input.reducedMotion) return { play: false, why: 'reduced_motion' };
  if (input.muted) return { play: false, why: 'muted' };
  if (input.hidden) return { play: false, why: 'hidden' };
  if (
    input.lastEffectAtMs !== null &&
    input.nowMs - input.lastEffectAtMs < COOLDOWN_MS
  ) {
    return { play: false, why: 'cooldown' };
  }
  return { play: true };
}

/** Whether the badge text should show. It shows for every burst, so a skipped burst is still visible as text. */
export function showsBadge(effect: Effect): boolean {
  return effect === 'crack_burst' || effect === 'laugh_burst';
}
