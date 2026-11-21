<script lang="ts">
  // Shared chrome for the signed-in app: the same approved graveyard world as the
  // landing and sign-in screens, dimmed hard so work surfaces stay readable, with
  // the Creepster wordmark as the way home.
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  let { children }: { children?: import('svelte').Snippet } = $props();
</script>

<main class="relative min-h-screen overflow-hidden bg-bg text-text">
  <img
    src="/art/landing/moon-graveyard-poster-960.webp"
    srcset="/art/landing/moon-graveyard-poster-960.webp 960w, /art/landing/moon-graveyard-poster-1920.webp 1920w"
    sizes="100vw"
    alt=""
    class="pointer-events-none fixed inset-0 h-full w-full object-cover opacity-40"
  />
  <div class="scrim pointer-events-none fixed inset-0"></div>

  <header class="relative border-b border-border/70 bg-bg/60 backdrop-blur-sm">
    <div
      class="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-y-1 px-4 py-3 sm:px-6"
    >
      <a
        href={resolve('/dashboard')}
        class="font-wordmark text-2xl tracking-wide text-text transition-colors hover:text-accent-candle-glow focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-focus"
      >
        Counterspell
      </a>
      <nav class="flex items-center gap-1 text-sm" aria-label="App">
        <a
          href={resolve('/problems')}
          class="rounded px-3 py-1.5 text-text-muted transition-colors hover:bg-surface hover:text-text focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
        >
          Problems
        </a>
        <a
          href={resolve('/break')}
          class="rounded px-3 py-1.5 text-text-muted transition-colors hover:bg-surface hover:text-text focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
        >
          Break room
        </a>
        <a
          href={resolve('/settings/account')}
          aria-current={page.url.pathname === '/settings/account'
            ? 'page'
            : undefined}
          class="rounded px-3 py-1.5 text-text-muted transition-colors hover:bg-surface hover:text-text focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus aria-[current=page]:bg-accent-moss-deep aria-[current=page]:text-text"
        >
          Settings
        </a>
      </nav>
    </div>
  </header>

  <div class="relative mx-auto max-w-5xl px-4 py-6 sm:px-6">
    {@render children?.()}
  </div>
</main>

<style>
  .scrim {
    background: radial-gradient(
      ellipse at center,
      rgba(8, 13, 14, 0.72) 0%,
      rgba(8, 13, 14, 0.92) 100%
    );
  }
</style>
