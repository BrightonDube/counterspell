<script>
  // Account settings (auth T32). The server decides what the account may do. This page shows each
  // action enabled or disabled from the capabilities that GET /api/me returns, and never checks
  // for the demo account itself.
  import { goto } from '$app/navigation';
  import { browser } from '$app/env';
  import { page } from '$app/state';
  import {
    AuthApiError,
    logoutEverywhere,
    requestEmailLink,
    startProviderLink
  } from '#lib/auth/client.js';
  import {
    errorFromQuery,
    linkedMessage,
    messageFor
  } from '#lib/auth/messages.js';
  import { session } from '#lib/auth/session.svelte.js';

  const ACCOUNT_PATH = '/settings/account';
  const DEMO_REASON = "The demo account can't be changed.";
  /** @type {Record<string, string>} */
  const PROVIDER_NAMES = { github: 'GitHub', google: 'Google' };

  // The query is read only in the browser; the prerendered page has none.
  const search = browser ? page.url.search : '';
  const linkedNote = linkedMessage(search);
  const errorNote = errorFromQuery(search);

  let problem = $state('');
  let notice = $state('');
  let email = $state('');
  let busy = $state(false);

  /** Sends the visitor to sign in, and brings them back to this page afterwards. */
  function signInAgain() {
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- a fixed path with an encoded return path, not a build-time route id
    goto(`/login?return_to=${encodeURIComponent(ACCOUNT_PATH)}`);
  }

  /** @param {unknown} failure */
  function handleFailure(failure) {
    if (failure instanceof AuthApiError && failure.status === 401) {
      signInAgain();
      return;
    }
    problem =
      failure instanceof AuthApiError
        ? messageFor(failure.code)
        : messageFor('internal_error');
  }

  // Loads the account once. A signed-out visitor is sent to sign in.
  const ready = browser
    ? session.load().then(() => {
        if (session.status === 'signed_out') signInAgain();
      })
    : null;

  /** @param {'github' | 'google'} provider */
  async function linkProvider(provider) {
    busy = true;
    problem = '';
    try {
      const { authorize_url } = await startProviderLink(provider);
      // The provider's own consent page. The server built the URL and bound it to this session.
      window.location.assign(authorize_url);
    } catch (failure) {
      handleFailure(failure);
    } finally {
      busy = false;
    }
  }

  /** @param {SubmitEvent} event */
  async function addEmail(event) {
    event.preventDefault();
    busy = true;
    problem = '';
    notice = '';
    try {
      await requestEmailLink(email);
      // The same sentence whether or not the address belongs to someone, so nothing is revealed.
      notice =
        'If that address is free, a confirmation link is on its way. Check your inbox.';
    } catch (failure) {
      handleFailure(failure);
    } finally {
      busy = false;
    }
  }

  async function signOutEverywhere() {
    busy = true;
    problem = '';
    try {
      await logoutEverywhere();
      session.clear();
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- a fixed path of this app
      await goto('/login');
    } catch (failure) {
      handleFailure(failure);
    } finally {
      busy = false;
    }
  }

  const buttonClass =
    'inline-flex items-center rounded-md border border-border-strong px-4 py-2 text-text hover:bg-raised focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus disabled:cursor-not-allowed disabled:text-text-subtle disabled:hover:bg-transparent';
  const fieldClass =
    'w-full rounded-md border border-border-strong bg-surface px-3 py-2 text-text placeholder:text-text-subtle focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus';
</script>

<svelte:head>
  <title>Account · Counterspell</title>
</svelte:head>

<main
  class="mx-auto flex min-h-screen max-w-2xl flex-col gap-8 bg-bg px-6 py-16 text-text"
>
  <h1 class="text-3xl font-semibold tracking-tight">Account</h1>

  <!-- Confirmations and errors from the redirects of the link flow. Text only. -->
  {#if linkedNote}
    <p class="text-success" role="status">{linkedNote}</p>
  {/if}
  {#if errorNote}
    <p class="text-danger" aria-live="polite">{errorNote}</p>
  {/if}
  <div
    aria-live="polite"
    class="min-h-6 text-danger"
    data-testid="account-problem"
  >
    {problem}
  </div>

  {#if ready}
    {#await ready then}
      {#if session.status === 'signed_in' && session.user}
        {@const user = session.user}
        {@const can = session.capabilities ?? {}}
        {@const reason = user.is_demo
          ? DEMO_REASON
          : 'This action is not available for your account.'}

        <section class="space-y-2" aria-labelledby="details-heading">
          <h2
            id="details-heading"
            class="text-sm font-medium uppercase tracking-wide text-text-subtle"
          >
            Your details
          </h2>
          <p>
            Email: <span class="text-text-muted"
              >{user.email ?? 'No email address yet'}</span
            >
          </p>
          <p>
            Linked sign-in:
            <span class="text-text-muted">
              {user.providers.length
                ? user.providers
                    .map(
                      (/** @type {string} */ name) =>
                        PROVIDER_NAMES[name] ?? name
                    )
                    .join(', ')
                : 'None'}
            </span>
          </p>
        </section>

        <section class="space-y-3" aria-labelledby="link-heading">
          <h2
            id="link-heading"
            class="text-sm font-medium uppercase tracking-wide text-text-subtle"
          >
            Link a sign-in method
          </h2>
          <div class="flex flex-wrap gap-3">
            <button
              class={buttonClass}
              type="button"
              disabled={!can.link_provider || busy}
              onclick={() => linkProvider('github')}
            >
              Link GitHub
            </button>
            <button
              class={buttonClass}
              type="button"
              disabled={!can.link_provider || busy}
              onclick={() => linkProvider('google')}
            >
              Link Google
            </button>
          </div>
          {#if !can.link_provider}
            <p class="text-text-muted">{reason}</p>
          {/if}
        </section>

        <section class="space-y-3" aria-labelledby="email-heading">
          <h2
            id="email-heading"
            class="text-sm font-medium uppercase tracking-wide text-text-subtle"
          >
            Add an email address
          </h2>
          {#if user.email}
            <p class="text-text-muted">
              This account already has an email address.
            </p>
          {:else}
            <form class="space-y-3" onsubmit={addEmail}>
              <div class="space-y-1">
                <label class="block text-sm text-text-muted" for="account-email"
                  >Email address</label
                >
                <input
                  id="account-email"
                  class={fieldClass}
                  type="email"
                  autocomplete="email"
                  required
                  disabled={!can.link_email}
                  bind:value={email}
                />
              </div>
              <button
                class={buttonClass}
                type="submit"
                disabled={!can.link_email || busy}
              >
                Send a confirmation link
              </button>
              {#if !can.link_email}
                <p class="text-text-muted">{reason}</p>
              {/if}
            </form>
          {/if}
          {#if notice}
            <p class="text-info" role="status">{notice}</p>
          {/if}
        </section>

        <section class="space-y-3" aria-labelledby="sessions-heading">
          <h2
            id="sessions-heading"
            class="text-sm font-medium uppercase tracking-wide text-text-subtle"
          >
            Sessions
          </h2>
          <button
            class={buttonClass}
            type="button"
            disabled={!can.logout_everywhere || busy}
            onclick={signOutEverywhere}
          >
            Sign out everywhere
          </button>
          {#if !can.logout_everywhere}
            <p class="text-text-muted">{reason}</p>
          {/if}
        </section>
      {/if}
    {:catch}
      <p class="text-danger">{messageFor('internal_error')}</p>
    {/await}
  {/if}
</main>
