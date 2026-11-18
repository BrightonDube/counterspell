<script lang="ts">
  // Shown for any route that fails or does not exist. It states what happened and offers one way back.
  // It never shows a mood, a verdict or a run result: those come only from measured runs.
  import { resolve } from '$app/paths';
  let { status }: { status: number } = $props();

  const copy = $derived(
    status === 404
      ? {
          heading: 'Page not found',
          body: 'We could not find that page. The link may be wrong, or the page may have moved.'
        }
      : {
          heading: 'Something went wrong',
          body: 'The problem is on our side. Try again in a moment.'
        }
  );
</script>

<section
  class="mx-auto flex max-w-xl flex-col items-start gap-6 px-6 py-24 sm:px-10"
>
  <img
    class="h-64 w-64 object-contain"
    src="/art/familiar/jack-o-lantern-orange.webp"
    width="256"
    height="256"
    alt=""
    aria-hidden="true"
    decoding="async"
  />
  <h1
    class="text-[2.5rem] leading-tight font-semibold tracking-[-0.01em] text-text"
  >
    {copy.heading}
  </h1>
  <p class="text-lg leading-8 text-text-muted">{copy.body}</p>
  <p class="font-mono text-sm text-text-subtle">Error {status}</p>
  <a
    href={resolve('/')}
    class="inline-flex items-center rounded bg-accent-candle px-6 py-3 text-base font-semibold text-bg transition-colors duration-200 hover:bg-accent-candle-glow focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-bg focus-visible:outline-none"
  >
    Back to the start
  </a>
</section>
