<script>
  // Sign-in page. The deployment offers provider sign-in only; the server reports which
  // providers are configured and a signed-in visitor is sent straight to their destination.
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { browser } from '$app/env';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { getConfig, providerSignInUrl } from '#lib/auth/client.js';
  import { errorFromQuery, messageFor } from '#lib/auth/messages.js';
  import { postAuthTarget, returnToFromQuery } from '#lib/auth/return-to.js';
  import { session } from '#lib/auth/session.svelte.js';
  import { WORDMARK, TAGLINE } from '#lib/landing/copy.js';

  // Only the browser reads the query and asks the server. The prerendered page has no query and no options.
  const config = browser ? getConfig() : null;

  const search = browser ? page.url.search : '';
  const returnTo = postAuthTarget(returnToFromQuery(search));
  const queryError = errorFromQuery(search);

  let problem = $state('');

  // A signed-in visitor never needs this page: send them on their way.
  onMount(async () => {
    try {
      await session.load();
    } catch {
      return;
    }
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- return_to is a user-chosen path, sanitised above; SvelteKit's resolve() only accepts build-time route ids
    if (session.status === 'signed_in') await goto(returnTo);
  });
</script>

<svelte:head>
  <title>Sign in · Counterspell</title>
  <meta
    name="description"
    content="Sign in to Counterspell to track your progress and pick up where you left off."
  />
</svelte:head>

<main
  class="relative flex min-h-screen flex-col overflow-hidden bg-bg text-text"
>
  <!-- The approved moon-and-graveyard still, shared with the landing title screen. -->
  <img
    src="/art/landing/moon-graveyard-poster-960.webp"
    srcset="/art/landing/moon-graveyard-poster-960.webp 960w, /art/landing/moon-graveyard-poster-1920.webp 1920w"
    sizes="100vw"
    alt=""
    class="pointer-events-none absolute inset-0 h-full w-full object-cover"
  />
  <!-- Readability scrim: darker at the edges so the panel carries the eye. -->
  <div class="scrim pointer-events-none absolute inset-0"></div>

  <div
    class="relative flex min-h-screen flex-col items-center justify-center px-6 py-16"
  >
    <a
      href={resolve('/')}
      class="font-wordmark text-5xl tracking-wide text-text transition-colors hover:text-accent-candle-glow focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-focus sm:text-6xl"
      aria-label="Counterspell — back to the front page"
    >
      {WORDMARK}
    </a>
    <p class="mt-3 text-center text-sm text-text-muted sm:text-base">
      {TAGLINE}
    </p>

    <section
      class="mt-10 w-full max-w-sm rounded-xl border border-border bg-surface/90 p-6 shadow-[0_24px_60px_-12px_rgba(0,0,0,0.75)] backdrop-blur-sm sm:p-8"
      aria-labelledby="signin-heading"
    >
      <h1
        id="signin-heading"
        class="text-xl font-semibold tracking-tight text-text"
      >
        Sign in
      </h1>
      <p class="mt-1 text-sm text-text-muted">Pick up where you left off.</p>

      <!-- Live region: error sentences from the server or the query string, read out by screen readers. -->
      <div
        aria-live="polite"
        class="mt-4 min-h-6 text-sm text-danger"
        data-testid="login-problem"
      >
        {#if queryError && !problem}{queryError}{:else}{problem}{/if}
      </div>

      {#if config}
        {#await config then options}
          <div class="mt-4 flex flex-col gap-3">
            {#if options.providers.github}
              <!-- eslint-disable svelte/no-navigation-without-resolve -- server sign-in endpoint, not a SvelteKit route -->
              <a
                href={providerSignInUrl('github', returnTo)}
                class="group inline-flex h-12 w-full items-center justify-center gap-3 rounded-full border border-border-strong bg-raised px-4 text-sm font-medium text-text transition-colors hover:border-accent-candle hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
              >
                <svg
                  viewBox="0 0 24 24"
                  class="h-5 w-5 fill-current text-text-muted transition-colors group-hover:text-text"
                  aria-hidden="true"
                >
                  <path
                    fill-rule="evenodd"
                    d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"
                  />
                </svg>
                Continue with GitHub
              </a>
              <!-- eslint-enable svelte/no-navigation-without-resolve -->
            {/if}
            {#if options.providers.google}
              <!-- eslint-disable svelte/no-navigation-without-resolve -- server sign-in endpoint, not a SvelteKit route -->
              <a
                href={providerSignInUrl('google', returnTo)}
                class="group inline-flex h-12 w-full items-center justify-center gap-3 rounded-full border border-border-strong bg-raised px-4 text-sm font-medium text-text transition-colors hover:border-accent-candle hover:bg-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
              >
                <svg viewBox="0 0 24 24" class="h-5 w-5" aria-hidden="true">
                  <path
                    fill="#4285F4"
                    d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                  />
                  <path
                    fill="#34A853"
                    d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                  />
                  <path
                    fill="#FBBC05"
                    d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
                  />
                  <path
                    fill="#EA4335"
                    d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
                  />
                </svg>
                Continue with Google
              </a>
              <!-- eslint-enable svelte/no-navigation-without-resolve -->
            {/if}
            {#if !options.providers.github && !options.providers.google}
              <p class="text-sm text-text-muted">
                Sign-in is not configured on this deployment yet.
              </p>
            {/if}
          </div>
        {:catch}
          <p class="mt-4 text-sm text-danger">
            {messageFor('internal_error')}
          </p>
        {/await}
      {/if}
    </section>
  </div>
</main>

<style>
  .scrim {
    background: radial-gradient(
      ellipse at center,
      rgba(8, 13, 14, 0.55) 0%,
      rgba(8, 13, 14, 0.88) 100%
    );
  }
</style>
