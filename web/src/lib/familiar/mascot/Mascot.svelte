<script lang="ts">
  // The mascot draws one static face for the measured mood. It never loads GSAP here; effects are
  // added in timelines.ts. Art is fetched from our own origin and drawn only if it meets the contract.
  import type { Mood } from '../types';
  import { planEffect, type Effect } from './plan';
  import { validate } from './contract';
  import { glowCss, moodStyle } from './mood-style';
  import { startBurst, startCrossfade } from './timelines';

  interface Props {
    mood: Mood;
    reason: string;
    /** Plain-English result sentence. Rendered as text only. */
    headline: string;
    badge: string;
    /** True when the measured run failed, so the mood text uses the danger colour. */
    failed?: boolean;
    effect?: Effect;
    live?: boolean;
    muted?: boolean;
    hidden?: boolean;
    reducedMotion?: boolean;
    onToggleHidden?: () => void;
    /** Seed for the burst particles. The page passes the run id so a replay looks the same. */
    seed?: number;
  }

  let {
    mood,
    reason,
    headline,
    badge,
    failed = false,
    hidden = false,
    effect: burstKind = 'none',
    live = false,
    muted = false,
    reducedMotion = false,
    onToggleHidden,
    seed = 1
  }: Props = $props();

  const ART_URL = '/familiar/jack-o-lantern.placeholder.svg';

  let face: HTMLDivElement | undefined = $state();
  let artDrawn = $state(false);
  let drawnInto: HTMLDivElement | null = null;
  /** When the last burst started, for the planner's 2 s cooldown. Not reactive on purpose. */
  let lastEffectAt: number | null = null;

  const style = $derived(moodStyle(mood));
  const glow = $derived(glowCss(style.glow));
  const liveText = $derived(`Familiar: ${style.name}. ${headline}`);

  /** Fetches the art, checks the id contract, and draws it with DOM calls only. */
  async function drawArt(target: HTMLDivElement): Promise<void> {
    const response = await fetch(ART_URL);
    if (!response.ok) return;
    const text = await response.text();
    if (!validate(text).ok) return;
    const doc = new DOMParser().parseFromString(text, 'image/svg+xml');
    const svg = doc.documentElement;
    if (svg.nodeName !== 'svg') return;
    target.replaceChildren(document.importNode(svg, true));
    artDrawn = true;
  }

  // Draw the art once per mounted face. A missing or invalid file leaves the badge text only.
  $effect(() => {
    if (face === undefined || face === drawnInto) return;
    drawnInto = face;
    drawArt(face).catch(() => undefined);
  });

  // Show only the eyes and mouth for the current mood; hide the other four layers of each kind.
  // A live change of mood fades the new expression in, unless motion is off.
  $effect(() => {
    if (!artDrawn || face === undefined) return;
    const shown = new Set([style.eyesLayer, style.mouthLayer]);
    for (const layer of face.querySelectorAll<SVGElement>(
      '[id^="fam-eyes-"], [id^="fam-mouth-"]'
    )) {
      layer.style.display = shown.has(layer.id) ? '' : 'none';
    }
    const root = face.querySelector<SVGSVGElement>('svg');
    if (root === null || !live || reducedMotion || muted) return;
    const fade = startCrossfade(root, [style.eyesLayer, style.mouthLayer]);
    return () => fade.dispose();
  });

  // Plays the burst only for a live result the planner allows. Any change in the inputs, or the
  // component leaving the page, disposes the burst that is running.
  $effect(() => {
    if (!artDrawn || face === undefined || burstKind === 'none') return;
    const root = face.querySelector<SVGSVGElement>('svg');
    if (root === null) return;
    const plan = planEffect({
      effect: burstKind,
      live,
      reducedMotion,
      muted,
      hidden,
      nowMs: performance.now(),
      lastEffectAtMs: lastEffectAt
    });
    if (!plan.play) return;
    lastEffectAt = performance.now();
    const burst = startBurst(root, burstKind, seed);
    return () => burst.dispose();
  });
</script>

<section aria-label="Efficiency Familiar" class="flex flex-col items-center gap-2">
  {#if hidden}
    <button
      type="button"
      aria-label="Show Familiar"
      class="flex min-h-11 min-w-11 items-center justify-center rounded"
      onclick={() => onToggleHidden?.()}
    >
      <span class="block h-4 w-4 rounded-full bg-accent-candle" aria-hidden="true"></span>
    </button>
  {:else}
    <div
      bind:this={face}
      class="h-40 w-40"
      style:--fam-glow={glow}
      data-mood={mood}
      data-reason={reason}
      aria-hidden="true"
    ></div>
    <p class="text-sm font-semibold capitalize {failed ? 'text-danger' : 'text-text'}">
      {style.name}
    </p>
    {#if badge !== ''}
      <span class="rounded bg-raised px-2 py-1 text-sm text-text-muted">{badge}</span>
    {/if}
    <p class="text-center text-sm text-text-muted">{headline}</p>
  {/if}
  <p aria-live="polite" class="sr-only">{liveText}</p>
</section>
