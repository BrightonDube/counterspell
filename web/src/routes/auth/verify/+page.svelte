<script>
  // Link confirmation page (auth T31). The token arrives in the URL fragment, which the browser
  // does not send to the server. The page takes it out of the address bar at once, keeps it in
  // memory only, and sends it only after the user presses the confirm button. Scanners that
  // open links therefore cannot use up the token.
  import { goto } from '$app/navigation';
  import { browser } from '$app/env';
  import {
    AuthApiError,
    requestMagicLink,
    verifyMagicLink
  } from '#lib/auth/client.js';
  import { messageFor } from '#lib/auth/messages.js';
  import { returnToFromQuery } from '#lib/auth/return-to.js';
  import { session } from '#lib/auth/session.svelte.js';

  // Read once at start-up. Only the browser has a fragment; the prerendered page has none.
  const fragmentToken = browser
    ? (new URLSearchParams(window.location.hash.slice(1)).get('token') ?? '')
    : '';
  if (browser) {
    window.history.replaceState(
      window.history.state,
      '',
      window.location.pathname + window.location.search
    );
  }

  /** Where the page is in its flow. */
  let stage = $state(browser && !fragmentToken ? 'missing' : 'ready');
  /** The one-time token. It lives only in this component and is never written to storage or logs. */
  let token = $state(fragmentToken);
  let problem = $state('');
  let email = $state('');
  let notice = $state('');

  async function confirm() {
    stage = 'working';
    problem = '';
    try {
      const result = await verifyMagicLink(token);
      token = '';
      await session.load();
      stage = 'done';
      const target = returnToFromQuery(
        `?return_to=${encodeURIComponent(result.return_to ?? '/')}`
      );
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- target is a sanitised user-chosen path, not a build-time route id
      await goto(target);
    } catch (failure) {
      if (failure instanceof AuthApiError && failure.status === 401) {
        // An email-linking link needs the account's own session, so the user signs in first.
        stage = 'needs_sign_in';
        problem = messageFor('unauthenticated');
        return;
      }
      token = '';
      stage = 'expired';
      problem =
        failure instanceof AuthApiError
          ? messageFor(failure.code)
          : messageFor('internal_error');
    }
  }

  /** @param {SubmitEvent} event */
  async function requestNewLink(event) {
    event.preventDefault();
    notice = '';
    problem = '';
    try {
      await requestMagicLink(email, '/');
      // The same sentence for every address, so the page cannot show who has an account.
      notice =
        'If that address can sign in, a new link is on its way. Check your inbox.';
    } catch (failure) {
      problem =
        failure instanceof AuthApiError
          ? messageFor(failure.code)
          : messageFor('internal_error');
    }
  }

  const buttonClass =
    'inline-flex items-center rounded-md bg-accent-candle px-4 py-2 font-medium text-bg hover:bg-accent-candle-glow focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus disabled:opacity-60';
  const fieldClass =
    'w-full rounded-md border border-border-strong bg-surface px-3 py-2 text-text placeholder:text-text-subtle focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus';
</script>

<svelte:head>
  <title>Confirm sign-in · Counterspell</title>
</svelte:head>

<main
  class="mx-auto flex min-h-screen max-w-md flex-col gap-8 bg-bg px-6 py-16 text-text"
>
  <h1 class="text-3xl font-semibold tracking-tight">Confirm sign-in</h1>

  <!-- Live region for status and errors. Text only; server messages are never rendered as HTML. -->
  <div
    aria-live="polite"
    class="min-h-6 text-danger"
    data-testid="verify-problem"
  >
    {problem}
  </div>

  {#if stage === 'ready' || stage === 'working' || stage === 'needs_sign_in'}
    <section class="space-y-4">
      <p class="text-text-muted">
        Press the button to finish signing in on this browser.
      </p>
      <button
        class={buttonClass}
        type="button"
        disabled={stage === 'working'}
        onclick={confirm}
      >
        Confirm sign-in
      </button>
      {#if stage === 'needs_sign_in'}
        <p class="text-text-muted">
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
          <a class="text-accent-moss underline underline-offset-4" href="/login"
            >Sign in first</a
          >, then open this link again.
        </p>
      {/if}
    </section>
  {:else if stage === 'done'}
    <p class="text-success" role="status">
      You are signed in. Taking you back now.
    </p>
  {:else if stage === 'missing' || stage === 'expired'}
    <section class="space-y-4">
      <p class="text-text-muted">
        Ask for a new link below. Each link works once and expires after 15
        minutes.
      </p>
      <form class="space-y-4" onsubmit={requestNewLink}>
        <div class="space-y-1">
          <label class="block text-sm text-text-muted" for="new-link-email"
            >Email address</label
          >
          <input
            id="new-link-email"
            class={fieldClass}
            type="email"
            autocomplete="email"
            required
            bind:value={email}
          />
        </div>
        <button class={buttonClass} type="submit">Send a new link</button>
      </form>
      {#if notice}
        <p class="text-info" role="status">{notice}</p>
      {/if}
    </section>
  {/if}
</main>
