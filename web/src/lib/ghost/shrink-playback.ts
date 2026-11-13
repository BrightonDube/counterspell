// Plans the playback of a recorded shrink trace. It only orders and times steps the server recorded.
// It never creates, interpolates or edits an input, and it never plays a trace whose last step
// disagrees with the ghost input.
import type { Json } from '../contract';
import type { ShrinkStepView, ShrinkTraceView } from './types';

/** The whole playback should finish within about this many milliseconds. */
export const PLAYBACK_BUDGET_MS = 3000;

/** The longest crossfade for one recorded step, in milliseconds. */
const MAX_STEP_MS = 250;

export type Trust = 'agrees' | 'disagrees' | 'no_steps';

export interface PlaybackPlan {
  mode: 'animated' | 'static';
  /** The accepted steps, in recorded order. Each is exactly what the server sent. */
  steps: ShrinkStepView[];
  /** Duplicate or out-of-order steps that were not played. */
  skipped: number;
  /** Time for one step's crossfade. Zero under reduced motion. */
  stepDelayMs: number;
  /** Steps the server summarised and did not send. The UI shows this count, never guessed steps. */
  omittedSteps: number;
  totalSteps: number;
  /** Whether the last recorded input equals the ghost input. */
  trust: Trust;
}

/** Serialises a JSON value with sorted object keys, so equal values compare equal. */
export function stableStringify(value: Json): string {
  if (Array.isArray(value)) {
    return `[${value.map(stableStringify).join(',')}]`;
  }
  if (value !== null && typeof value === 'object') {
    const entries = Object.keys(value)
      .sort()
      .map(
        (key) => `${JSON.stringify(key)}:${stableStringify(value[key] as Json)}`
      );
    return `{${entries.join(',')}}`;
  }
  return JSON.stringify(value);
}

/**
 * Accepts steps whose index is greater than every step before them, which drops duplicates and
 * out-of-order arrivals. Then checks that the last accepted input is the ghost input.
 */
export function planPlayback(
  trace: ShrinkTraceView,
  ghostInput: Json,
  reducedMotion: boolean
): PlaybackPlan {
  const steps: ShrinkStepView[] = [];
  let skipped = 0;
  let lastIndex = 0;
  for (const step of trace.steps) {
    if (step.index > lastIndex) {
      steps.push(step);
      lastIndex = step.index;
    } else {
      skipped += 1;
    }
  }

  const last = steps[steps.length - 1];
  let trust: Trust = 'no_steps';
  if (last !== undefined) {
    trust =
      stableStringify(last.input) === stableStringify(ghostInput)
        ? 'agrees'
        : 'disagrees';
  }
  const shown = trust === 'disagrees' ? [] : steps;
  const stepDelayMs =
    reducedMotion || shown.length === 0
      ? 0
      : Math.min(MAX_STEP_MS, Math.floor(PLAYBACK_BUDGET_MS / shown.length));

  return {
    mode: reducedMotion ? 'static' : 'animated',
    steps: shown,
    skipped,
    stepDelayMs,
    omittedSteps: trace.omittedSteps,
    totalSteps: trace.totalSteps,
    trust
  };
}
