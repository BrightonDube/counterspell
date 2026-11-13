<svelte:head>
  <title>Counterspell</title>
  <meta name="description" content="Practice code with a ghost at your side." />
</svelte:head>

<main
  class="min-h-screen bg-bg px-6 py-16 text-text sm:px-10"
  data-smoke-marker="counterspell-svelte-static"
>
  <div class="mx-auto flex max-w-3xl flex-col gap-8">
    <p class="font-mono text-sm uppercase tracking-[0.2em] text-accent-candle">
      Counterspell static build
    </p>
    <div class="max-w-2xl space-y-5">
      <h1 class="text-4xl font-semibold tracking-tight text-text sm:text-6xl">
        Learn why your code breaks.
      </h1>
      <p class="max-w-xl text-lg leading-8 text-text-muted">
        The ghost engine and trusted judge are coming soon. This calm page
        proves the SvelteKit build can be served by the Rust process.
      </p>
    </div>
    <div
      class="w-fit rounded border border-border bg-surface px-4 py-3 text-sm text-text-subtle"
    >
      Skeleton milestone · judging is not implemented yet
    </div>
  </div>
</main>
