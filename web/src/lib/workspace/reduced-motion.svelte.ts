// Tracks the learner's prefers-reduced-motion setting so animated regions can go
// static. Tests inject an initial value; the browser keeps it live.
import { browser } from '$app/env';

/** Returns live reduced-motion state. `initial` pins the value for tests. */
export function createReducedMotion(initial?: boolean): {
  readonly value: boolean;
} {
  let value = $state(initial ?? false);

  $effect(() => {
    if (!browser || initial !== undefined) return;
    if (typeof globalThis.matchMedia !== 'function') return;
    const query = globalThis.matchMedia('(prefers-reduced-motion: reduce)');
    value = query.matches;
    const onChange = (event: MediaQueryListEvent) => {
      value = event.matches;
    };
    query.addEventListener('change', onChange);
    return () => query.removeEventListener('change', onChange);
  });

  return {
    get value() {
      return value;
    }
  };
}
