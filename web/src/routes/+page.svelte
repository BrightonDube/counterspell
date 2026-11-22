<script lang="ts">
  import { onMount } from 'svelte';
  import { startScenes } from '../lib/landing/scenes';
  import { planLanding, type LandingPlan } from '../lib/landing/motion-plan';
  import GhostBeats from '../lib/landing/GhostBeats.svelte';
  import Hero from '../lib/landing/Hero.svelte';
  import SignUpCta from '../lib/landing/SignUpCta.svelte';
  import {
    BENEFITS,
    HEADLINE,
    INTRO_BODY,
    SUPPORT_LINE
  } from '../lib/landing/copy';
  import {
    HERO_POSTER_SIZES,
    HERO_POSTER_SRCSET
  } from '../lib/landing/hero-media';

  // Vite fingerprints the font, so this URL matches the one the @font-face rule in app.css uses.
  import creepsterUrl from '../../static/fonts/Creepster-Regular.woff2?url';

  // The static HTML already shows every word. The plan only decides what motion to add.
  let plan = $state.raw<LandingPlan>(
    planLanding({
      reducedMotion: false,
      viewportWidth: 1280,
      scriptsRunning: false
    })
  );

  onMount(() => {
    const reducedMotion = window.matchMedia(
      '(prefers-reduced-motion: reduce)'
    ).matches;
    const next = planLanding({
      reducedMotion,
      viewportWidth: window.innerWidth,
      scriptsRunning: true
    });
    plan = next;
    if (!next.loadGsap) return;

    // GSAP loads after a delay, so the page may be left first. Dispose whichever comes last.
    let active = true;
    let dispose: (() => void) | undefined;
    startScenes(next)
      .then((revert) => {
        if (active) dispose = revert;
        else revert();
      })
      .catch(() => {
        // Motion is optional. The static page already shows every word and beat.
      });
    return () => {
      active = false;
      dispose?.();
    };
  });
</script>

<svelte:head>
  <title>Counterspell — Sharpen Your Mind with Coding Challenges</title>
  <link
    rel="preload"
    href={creepsterUrl}
    as="font"
    type="font/woff2"
    crossorigin="anonymous"
  />
  <link
    rel="preload"
    as="image"
    type="image/avif"
    imagesrcset={HERO_POSTER_SRCSET}
    imagesizes={HERO_POSTER_SIZES}
    fetchpriority="high"
  />

  <meta
    name="description"
    content="Challenge your thinking with coding problems, sharpen your debugging skills, and learn from mistakes. Counterspell helps you become a stronger independent problem solver."
  />
</svelte:head>

<main
  class="min-h-screen bg-bg text-text"
  lang="en"
  data-smoke-marker="counterspell-svelte-static"
  data-motion={plan.mode}
>
  <Hero {plan} />
  <div class="mx-auto flex max-w-3xl flex-col gap-16 px-6 py-16 sm:px-10">
    <div class="flex flex-col gap-4">
      <h2
        class="text-3xl leading-tight font-semibold tracking-[-0.01em] text-text sm:text-4xl"
      >
        {HEADLINE}
      </h2>
      <p class="max-w-2xl text-lg leading-8 text-text-muted">{SUPPORT_LINE}</p>
      <p class="max-w-2xl text-base leading-7 text-text-muted">{INTRO_BODY}</p>
      <ul class="grid max-w-2xl gap-4 sm:grid-cols-3">
        {#each BENEFITS as benefit (benefit.heading)}
          <li>
            <h3 class="font-semibold text-text">{benefit.heading}</h3>
            <p class="mt-1 text-sm leading-6 text-text-muted">{benefit.body}</p>
          </li>
        {/each}
      </ul>
    </div>
    <div data-reveal>
      <GhostBeats />
    </div>
    <div data-reveal>
      <SignUpCta />
    </div>
  </div>
</main>
