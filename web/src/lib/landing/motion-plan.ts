// Decides what the landing page animates. Pure: no DOM, no GSAP, no window.
// The page passes in the facts it can read and gets back a plan.

/** Viewports narrower than this (CSS px) keep the far layer and bats static. */
const PHONE_MAX_WIDTH = 640;

export interface MotionFacts {
  /** True when the visitor has asked for reduced motion. */
  reducedMotion: boolean;
  /** Viewport width in CSS pixels. */
  viewportWidth: number;
  /** True once the page's scripts are running (set in onMount). */
  scriptsRunning: boolean;
}

export interface LandingPlan {
  mode: 'motion' | 'static';
  /** Only true in motion mode. GSAP is never imported otherwise. */
  loadGsap: boolean;
  /** Which scene layers are shown. The headline and beats are never layers. */
  layers: { moon: boolean; far: boolean; near: boolean; smoke: boolean };
  /** The one-shot bat flight. */
  bats: boolean;
  /** Scroll reveals for the beats and the call to action. */
  reveals: boolean;
}

/**
 * Turns the visitor's facts into a plan for the landing scenes.
 *
 * Static mode shows every scene layer at its final position and starts no animation.
 * Motion mode turns the full effect on, except that narrow phones skip the far layer and bats.
 */
export function planLanding(facts: MotionFacts): LandingPlan {
  if (facts.reducedMotion || !facts.scriptsRunning) {
    return {
      mode: 'static',
      loadGsap: false,
      layers: { moon: true, far: true, near: true, smoke: false },
      bats: false,
      reveals: false
    };
  }

  const phone = facts.viewportWidth < PHONE_MAX_WIDTH;
  return {
    mode: 'motion',
    loadGsap: true,
    layers: { moon: true, far: !phone, near: true, smoke: true },
    bats: !phone,
    reveals: true
  };
}
