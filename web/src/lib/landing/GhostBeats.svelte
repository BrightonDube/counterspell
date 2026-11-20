<script lang="ts">
  import { BEATS } from './copy';
  import GhostExample from './GhostExample.svelte';

  // Each beat has one decorative picture from the approved art. The heading and text carry the
  // meaning, so the picture is hidden from assistive technology.
  const art = {
    Submit: 'crypt-key',
    Hunt: 'crystal-ball',
    Understand: 'brain-jar'
  } as const;

  // The icons show at 64 px, so a 128 px file covers a 2x screen. The 512 px file stays as the fallback.
  const src = (name: string) => `/art/objects/${name}.webp`;
  const srcset = (name: string) =>
    `/art/objects/${name}-128.webp 128w, ${src(name)} 512w`;
</script>

<div class="flex flex-col gap-12">
  {#each BEATS as beat (beat.heading)}
    <section class="flex flex-col gap-4">
      <div class="flex items-center gap-4">
        <img
          class="h-16 w-16 shrink-0 object-contain"
          src={src(art[beat.heading])}
          srcset={srcset(art[beat.heading])}
          sizes="64px"
          width="64"
          height="64"
          alt=""
          aria-hidden="true"
          loading="lazy"
          decoding="async"
        />
        <h3 class="text-xl font-semibold tracking-[-0.01em] text-text">
          {beat.heading}
        </h3>
      </div>
      <p class="max-w-xl text-base leading-7 text-text-muted">{beat.body}</p>
      {#if beat.heading === 'Hunt'}
        <GhostExample />
      {/if}
    </section>
  {/each}
</div>
