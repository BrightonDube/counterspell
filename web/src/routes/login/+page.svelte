<script lang="ts">
  import { goto } from '$app/navigation';
  import { browser } from '$app/env';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import { onMount, tick } from 'svelte';
  import {
    AuthApiError,
    getConfig,
    providerSignInUrl
  } from '#lib/auth/client.js';
  import { errorFromQuery } from '#lib/auth/messages.js';
  import {
    postAuthTarget,
    returnToFromQuery,
    safeReturnTo
  } from '#lib/auth/return-to.js';
  import {
    authProblem,
    signInWithPasskey,
    signInWithPassword,
    supportsPasskeys,
    verifyTwoFactor
  } from '#lib/auth/security.ts';
  import SecurityIcon from '#lib/auth/SecurityIcon.svelte';
  import { session } from '#lib/auth/session.svelte.js';
  import { WORDMARK, TAGLINE } from '#lib/landing/copy.js';

  type AuthOptions = {
    providers: { github: boolean; google: boolean };
    demo?: { enabled: boolean; email: string | null };
  };

  let search = $derived(browser ? page.url.search : '');
  let returnTo = $derived(postAuthTarget(returnToFromQuery(search)));
  let queryError = $derived(errorFromQuery(search));
  let twoFactor = $derived(new URLSearchParams(search).get('step') === '2fa');

  let problem = $state('');
  let busy = $state(false);
  let passkeysSupported = $state(false);
  let options = $state<AuthOptions | null>(null);
  let loadingOptions = $state(true);
  let email = $state('');
  let password = $state('');
  let code = $state('');
  let recovery = $state(false);
  let expired = $state(false);
  let codeInput = $state<HTMLInputElement>();

  const choiceClass =
    'inline-flex min-h-12 w-full items-center justify-center gap-3 rounded-full border border-accent-moss-muted bg-surface px-4 py-2 text-sm font-medium text-text transition-colors hover:border-accent-moss-strong hover:bg-raised disabled:opacity-55';
  const primaryClass =
    'inline-flex min-h-12 w-full items-center justify-center gap-2 rounded-full border border-accent-candle bg-accent-candle px-4 py-2 text-sm font-semibold text-bg transition-colors hover:bg-accent-candle-glow disabled:opacity-55';
  const fieldClass =
    'w-full rounded-md border border-accent-moss-muted bg-bg px-3 py-2.5 text-base text-text placeholder:text-text-subtle caret-accent-candle';
  const textAction =
    'inline-flex min-h-11 items-center text-sm font-medium text-accent-moss-strong underline underline-offset-4 hover:text-text';

  onMount(() => {
    passkeysSupported = supportsPasskeys();
    void loadOptions();
    if (twoFactor) {
      void focusCode();
    } else {
      void session
        .load()
        .then(() => {
          if (session.status === 'signed_in' && !busy && !twoFactor) {
            return goToDestination(returnTo);
          }
        })
        .catch(() => {});
    }
  });

  async function loadOptions() {
    loadingOptions = true;
    try {
      options = await getConfig();
      if (options?.demo?.enabled && options.demo.email && !email)
        email = options.demo.email;
    } catch (failure) {
      if (!twoFactor) problem = authProblem(failure);
    } finally {
      loadingOptions = false;
    }
  }

  function goToDestination(target: string) {
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- a sanitized site-relative destination
    return goto(postAuthTarget(safeReturnTo(target)), { replaceState: true });
  }

  async function focusCode() {
    await tick();
    codeInput?.focus();
  }

  async function showTwoFactor() {
    code = '';
    recovery = false;
    expired = false;
    const query = new URLSearchParams({ step: '2fa', return_to: returnTo });
    await goto(resolve(`/login?${query}`), {
      replaceState: true
    });
    await focusCode();
  }

  async function startAgain() {
    code = '';
    recovery = false;
    expired = false;
    problem = '';
    await goto(resolve(`/login?return_to=${encodeURIComponent(returnTo)}`), {
      replaceState: true
    });
  }

  async function toggleRecovery() {
    recovery = !recovery;
    code = '';
    problem = '';
    await focusCode();
  }

  async function run(action: () => Promise<void>) {
    if (busy) return;
    busy = true;
    problem = '';
    try {
      await action();
    } catch (failure) {
      problem = authProblem(failure);
      expired =
        twoFactor &&
        failure instanceof AuthApiError &&
        (failure.code === 'security_challenge_expired' ||
          failure.code === 'unauthenticated');
    } finally {
      busy = false;
    }
  }

  async function passkeySignIn() {
    const result = await signInWithPasskey(returnTo);
    if (result.two_factor_required) await showTwoFactor();
    else await goToDestination(result.return_to ?? returnTo);
  }

  async function demoSignIn() {
    const result = await signInWithPassword(email.trim(), password, returnTo);
    password = '';
    if (result.two_factor_required) await showTwoFactor();
    else await goToDestination(result.return_to ?? returnTo);
  }

  async function verifyCode() {
    const result = await verifyTwoFactor(code.trim());
    code = '';
    await goToDestination(result.return_to);
  }
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
    class="relative flex min-h-screen flex-col items-center justify-center px-4 py-10 sm:px-6 sm:py-16"
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
      class="mt-8 w-full max-w-sm rounded-xl border border-border-strong bg-surface p-5 shadow-[0_24px_60px_-12px_rgba(0,0,0,0.75)] sm:p-7"
      aria-labelledby="signin-heading"
    >
      <h1
        id="signin-heading"
        class="text-xl font-semibold tracking-tight text-text"
      >
        {twoFactor ? 'Verify your sign-in' : 'Sign in'}
      </h1>

      {#if problem || queryError}
        <p
          role="alert"
          class="mt-3 text-sm text-danger"
          data-testid="login-problem"
        >
          {problem || queryError}
        </p>
      {/if}

      {#if twoFactor}
        {#if !expired}
          <p class="mt-2 text-sm text-text-muted">
            {recovery
              ? 'Enter one of the recovery codes you saved when you set up two-factor authentication.'
              : 'Enter the current 6-digit code from your authenticator app.'}
          </p>
          <form
            class="mt-5"
            onsubmit={(event) => {
              event.preventDefault();
              void run(verifyCode);
            }}
          >
            <label for="sign-in-code" class="mb-1.5 block text-sm font-medium">
              {recovery ? 'Recovery code' : 'Authenticator code'}
            </label>
            <input
              id="sign-in-code"
              class={fieldClass}
              type="text"
              inputmode={recovery ? 'text' : 'numeric'}
              pattern={recovery ? undefined : '[0-9]{6}'}
              maxlength={recovery ? 64 : 6}
              autocomplete={recovery ? 'off' : 'one-time-code'}
              autocapitalize="off"
              spellcheck={false}
              required
              bind:value={code}
              bind:this={codeInput}
            />
            <button
              class={primaryClass + ' mt-4'}
              type="submit"
              disabled={busy}
            >
              {busy ? 'Verifying…' : 'Verify and sign in'}
            </button>
          </form>
          <button
            class={textAction + ' mt-2'}
            type="button"
            disabled={busy}
            onclick={toggleRecovery}
          >
            {recovery ? 'Use my authenticator app' : 'Use a recovery code'}
          </button>
        {/if}
        <button
          class={expired ? primaryClass + ' mt-4' : textAction + ' block'}
          type="button"
          disabled={busy}
          onclick={startAgain}
        >
          {expired ? 'Start sign-in again' : 'Back to sign-in'}
        </button>
      {:else}
        <div class="mt-5 flex flex-col gap-2.5">
          {#if passkeysSupported}
            <button
              type="button"
              class={primaryClass}
              disabled={busy}
              onclick={() => run(passkeySignIn)}
            >
              <SecurityIcon name="key" />
              {busy ? 'Please wait…' : 'Use a passkey'}
            </button>
          {:else if !loadingOptions}
            <p class="text-sm text-text-muted">
              Passkeys are unavailable in this browser.
            </p>
          {/if}
          {#if options?.providers.github}
            <!-- eslint-disable svelte/no-navigation-without-resolve -- server sign-in endpoint, not a SvelteKit route -->
            <a
              href={providerSignInUrl('github', returnTo)}
              class={'group ' + choiceClass}
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
          {#if options?.providers.google}
            <!-- eslint-disable svelte/no-navigation-without-resolve -- server sign-in endpoint, not a SvelteKit route -->
            <a href={providerSignInUrl('google', returnTo)} class={choiceClass}>
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
          {#if loadingOptions}
            <p class="text-sm text-text-muted" role="status">
              Loading sign-in options…
            </p>
          {:else if !options}
            <button
              type="button"
              class={textAction}
              onclick={() => {
                problem = '';
                void loadOptions();
              }}
            >
              Reload sign-in options
            </button>
          {/if}
        </div>

        {#if options?.demo?.enabled}
          <details class="mt-4 border-t border-border pt-3">
            <summary
              class="cursor-pointer py-2 text-sm font-medium text-accent-moss-strong"
            >
              Use the demo account
            </summary>
            <form
              class="mt-2 space-y-3"
              onsubmit={(event) => {
                event.preventDefault();
                void run(demoSignIn);
              }}
            >
              <div>
                <label for="sign-in-demo-email" class="mb-1.5 block text-sm">
                  Demo email
                </label>
                <input
                  id="sign-in-demo-email"
                  class={fieldClass}
                  type="email"
                  autocomplete="username"
                  required
                  bind:value={email}
                />
              </div>
              <div>
                <label for="sign-in-demo-password" class="mb-1.5 block text-sm">
                  Demo password
                </label>
                <input
                  id="sign-in-demo-password"
                  class={fieldClass}
                  type="password"
                  autocomplete="current-password"
                  required
                  bind:value={password}
                />
              </div>
              <button type="submit" class={choiceClass} disabled={busy}>
                Sign in as the demo account
              </button>
            </form>
          </details>
        {/if}
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
