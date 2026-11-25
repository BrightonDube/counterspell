<script lang="ts">
  // Reusable animated-art wrapper: any shipped asset (static .webp or an animated .webm
  // loop with a static poster) gets a consistent motion vocabulary — `motion` picks the
  // idle animation, `bounce` adds the hover bob, `centre` margin-centres it in its block.
  // Keyframes live in app.css so every component shares the same motion language.
  // The video element is only created while the art is visible and full motion is allowed —
  // off-screen or reduced-motion visits never fetch the loop at all (WP05).
  import { onMount } from 'svelte';

  let {
    src,
    webm = '',
    poster = src,
    srcset = undefined,
    sizes = undefined,
    alt = '',
    size = 96,
    motion = 'breathe',
    bounce = true,
    centre = false
  }: {
    src: string;
    webm?: string;
    poster?: string;
    srcset?: string;
    sizes?: string;
    alt?: string;
    size?: number;
    motion?: 'breathe' | 'flame' | 'still';
    bounce?: boolean;
    centre?: boolean;
  } = $props();

  let host = $state<HTMLDivElement | undefined>();
  let video = $state<HTMLVideoElement | undefined>();
  let visible = $state(false);
  let reducedMotion = $state(false);

  onMount(() => {
    const media = window.matchMedia('(prefers-reduced-motion: reduce)');
    reducedMotion = media.matches;
    const onMotion = (event: MediaQueryListEvent) => {
      reducedMotion = event.matches;
    };
    media.addEventListener('change', onMotion);
    if (typeof IntersectionObserver === 'undefined') {
      // Environments without the API (tests, very old browsers) treat the art
      // as visible — the media behaves exactly as before.
      visible = true;
      return () => media.removeEventListener('change', onMotion);
    }
    // The loop attaches while the art is on screen and pauses when it leaves,
    // so no bytes or decode time are spent on media nobody can see.
    const observer = new IntersectionObserver((entries) => {
      if (entries[0]?.isIntersecting) {
        visible = true;
        void video?.play().catch(() => {});
      } else {
        video?.pause();
      }
    });
    if (host) observer.observe(host);
    return () => {
      media.removeEventListener('change', onMotion);
      observer.disconnect();
    };
  });
</script>

<div
  bind:this={host}
  aria-hidden="true"
  class="art"
  class:breathing={motion === 'breathe'}
  class:flicker={motion === 'flame'}
  class:bounceable={bounce}
  class:centred={centre}
  style="width: {size}px; height: {size}px"
>
  {#if webm && visible && !reducedMotion}
    <video
      bind:this={video}
      autoplay
      loop
      muted
      playsinline
      width={size}
      height={size}
      class="h-full w-full object-contain"
      aria-hidden="true"
      {poster}
    >
      <source src={webm} type="video/webm" />
      <img
        {src}
        {srcset}
        {sizes}
        width={size}
        height={size}
        {alt}
        aria-hidden="true"
        loading="lazy"
        decoding="async"
        class="h-full w-full object-contain"
      />
    </video>
  {:else}
    <img
      {src}
      {srcset}
      {sizes}
      width={size}
      height={size}
      {alt}
      aria-hidden="true"
      loading="lazy"
      decoding="async"
      class="h-full w-full object-contain"
    />
  {/if}
</div>

<style>
  .art {
    display: inline-block;
    transform-origin: 50% 90%;
  }
  .centred {
    display: block;
    margin-left: auto;
    margin-right: auto;
  }
  .breathing {
    animation: art-breathe 3.6s ease-in-out infinite;
  }
  .flicker {
    transform-origin: 50% 92%;
    animation: art-flicker 2.4s ease-in-out infinite;
  }
  .bounceable:hover {
    animation: art-bounce 0.6s ease-in-out;
  }
  @media (prefers-reduced-motion: reduce) {
    .art,
    .art:hover {
      animation: none;
    }
  }
</style>
