<script>
  // Sign-in page (auth T30). It offers only the sign-in methods the server reports as enabled.
  // Passwords are sent once, to the login endpoint, and are never stored or prefilled.
  import { goto } from '$app/navigation';
  import { browser } from '$app/env';
  import { page } from '$app/state';
  import { Tabs } from 'bits-ui';
  import {
    AuthApiError,
    getConfig,
    login,
    providerSignInUrl,
    requestMagicLink
  } from '#lib/auth/client.js';
  import { errorFromQuery, messageFor } from '#lib/auth/messages.js';
  import { returnToFromQuery } from '#lib/auth/return-to.js';
  import { session } from '#lib/auth/session.svelte.js';

  // Only the browser reads the query and asks the server. The prerendered page has no query and no options.
  const config = browser ? getConfig() : null;

  const search = browser ? page.url.search : '';
  const returnTo = returnToFromQuery(search);
  const queryError = errorFromQuery(search);

  let mode = $state('magic');
  let email = $state('');
  let password = $state('');
  let notice = $state('');
  let problem = $state('');
  let busy = $state(false);

  /** @param {unknown} failure */
  function sentenceFor(failure) {
    return failure instanceof AuthApiError
      ? messageFor(failure.code)
      : messageFor('internal_error');
  }

  /** @param {SubmitEvent} event */
  async function sendLink(event) {
    event.preventDefault();
    busy = true;
    problem = '';
    try {
      await requestMagicLink(email, returnTo);
      // The same sentence for every address, so the page cannot show who has an account.
      notice =
        'If that address can sign in, a link is on its way. Check your inbox.';
    } catch (failure) {
      problem = sentenceFor(failure);
    } finally {
      busy = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function signIn(event) {
    event.preventDefault();
    busy = true;
    problem = '';
    try {
      await login(email, password);
      password = '';
      await session.load();
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- return_to is a user-chosen path, sanitised above; SvelteKit's resolve() only accepts build-time route ids
      await goto(returnTo);
    } catch (failure) {
      password = '';
      problem = sentenceFor(failure);
    } finally {
      busy = false;
    }
  }

  /** @param {{ demo: { email: string } }} options */
  function openDemo(options) {
    mode = 'password';
    email = options.demo.email;
    password = '';
    problem = '';
    notice = '';
  }

  const fieldClass =
    'w-full rounded-md border border-border-strong bg-surface px-3 py-2 text-text placeholder:text-text-subtle focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus';
  const buttonClass =
    'inline-flex items-center rounded-md bg-accent-candle px-4 py-2 font-medium text-bg hover:bg-accent-candle-glow focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus disabled:opacity-60';
  const linkClass =
    'inline-flex items-center rounded-md border border-border-strong px-4 py-2 text-text hover:bg-raised focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus';
</script>

<svelte:head>
  <title>Sign in · Counterspell</title>
</svelte:head>

<main
  class="mx-auto flex min-h-screen max-w-md flex-col gap-8 bg-bg px-6 py-16 text-text"
>
  <header class="space-y-2">
    <h1 class="text-3xl font-semibold tracking-tight">Sign in</h1>
    <p class="text-text-muted">Pick up where you left off.</p>
  </header>

  <!-- Live region: error sentences from the server or the query string, read out by screen readers. -->
  <div
    aria-live="polite"
    class="min-h-6 text-danger"
    data-testid="login-problem"
  >
    {#if queryError && !problem}{queryError}{:else}{problem}{/if}
  </div>

  {#if config}
    {#await config then options}
      <section class="space-y-3" aria-labelledby="providers-heading">
        <h2
          id="providers-heading"
          class="text-sm font-medium uppercase tracking-wide text-text-subtle"
        >
          Continue with
        </h2>
        {#if options.providers.github}
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- server sign-in endpoint, not a SvelteKit route -->
          <a class={linkClass} href={providerSignInUrl('github', returnTo)}
            >GitHub</a
          >
        {/if}
        {#if options.providers.google}
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- server sign-in endpoint, not a SvelteKit route -->
          <a class={linkClass} href={providerSignInUrl('google', returnTo)}
            >Google</a
          >
        {/if}
      </section>

      {#if options.email.magic_link || options.email.password}
        <section class="space-y-4" aria-labelledby="email-heading">
          <h2
            id="email-heading"
            class="text-sm font-medium uppercase tracking-wide text-text-subtle"
          >
            Email
          </h2>
          <Tabs.Root bind:value={mode}>
            <Tabs.List class="mb-4 flex gap-2 border-b border-border">
              {#if options.email.magic_link}
                <Tabs.Trigger
                  value="magic"
                  class="px-3 py-2 text-text-muted data-[state=active]:border-b-2 data-[state=active]:border-accent-moss data-[state=active]:text-text focus-visible:outline-2 focus-visible:outline-focus"
                >
                  Email me a link
                </Tabs.Trigger>
              {/if}
              {#if options.email.password}
                <Tabs.Trigger
                  value="password"
                  class="px-3 py-2 text-text-muted data-[state=active]:border-b-2 data-[state=active]:border-accent-moss data-[state=active]:text-text focus-visible:outline-2 focus-visible:outline-focus"
                >
                  Password
                </Tabs.Trigger>
              {/if}
            </Tabs.List>

            {#if options.email.magic_link}
              <Tabs.Content value="magic">
                {#if notice && mode === 'magic'}
                  <p class="mb-3 text-info">{notice}</p>
                {/if}
                <form class="space-y-4" onsubmit={sendLink}>
                  <div class="space-y-1">
                    <label
                      class="block text-sm text-text-muted"
                      for="magic-email">Email address</label
                    >
                    <input
                      id="magic-email"
                      class={fieldClass}
                      type="email"
                      autocomplete="email"
                      required
                      bind:value={email}
                    />
                  </div>
                  <button class={buttonClass} type="submit" disabled={busy}
                    >Send me a link</button
                  >
                </form>
              </Tabs.Content>
            {/if}

            {#if options.email.password}
              <Tabs.Content value="password">
                <form class="space-y-4" onsubmit={signIn}>
                  <div class="space-y-1">
                    <label
                      class="block text-sm text-text-muted"
                      for="password-email">Email address</label
                    >
                    <input
                      id="password-email"
                      class={fieldClass}
                      type="email"
                      autocomplete="email"
                      required
                      bind:value={email}
                    />
                  </div>
                  <div class="space-y-1">
                    <label
                      class="block text-sm text-text-muted"
                      for="password-value">Password</label
                    >
                    <input
                      id="password-value"
                      class={fieldClass}
                      type="password"
                      autocomplete="current-password"
                      required
                      bind:value={password}
                    />
                  </div>
                  <button class={buttonClass} type="submit" disabled={busy}
                    >Sign in</button
                  >
                </form>
              </Tabs.Content>
            {/if}
          </Tabs.Root>
        </section>
      {/if}

      {#if options.demo.enabled}
        <section class="border-t border-border pt-6">
          <button
            type="button"
            class="text-accent-moss underline underline-offset-4 hover:text-accent-moss-strong focus-visible:outline-2 focus-visible:outline-focus"
            onclick={() => openDemo(options)}
          >
            Judging? Use the demo account
          </button>
        </section>
      {/if}
    {:catch}
      <p class="text-danger">{messageFor('internal_error')}</p>
    {/await}
  {/if}
</main>
