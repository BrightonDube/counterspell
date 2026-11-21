<script lang="ts">
  import { goto, replaceState } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { onMount } from 'svelte';
  import { messageFor } from '#lib/auth/messages.js';
  import { postAuthTarget, safeReturnTo } from '#lib/auth/return-to.js';
  import { authProblem, verifyEmailLink } from '#lib/auth/security.ts';

  let token = $state<string | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  let problem = $state('');

  onMount(() => {
    token = new URLSearchParams(window.location.hash.slice(1)).get('token');
    if (window.location.hash) replaceState(resolve('/auth/verify'), {});
    if (!token) problem = messageFor('invalid_or_expired_token');
    ready = true;
  });

  async function verify() {
    if (!token || busy) return;
    busy = true;
    problem = '';
    try {
      const result = await verifyEmailLink(token);
      token = null;
      if (result.status === 'email_linked') {
        await goto(resolve('/settings/account'), { replaceState: true });
      } else {
        // eslint-disable-next-line svelte/no-navigation-without-resolve -- a sanitized server destination
        await goto(postAuthTarget(safeReturnTo(result.return_to)), {
          replaceState: true
        });
      }
    } catch (failure) {
      problem = authProblem(failure);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:head>
  <title>Confirm sign-in link · Counterspell</title>
</svelte:head>

<main class="grid min-h-screen place-items-center bg-bg px-4 py-10 text-text">
  <section
    class="w-full max-w-sm rounded-xl border border-border-strong bg-surface p-6"
    aria-labelledby="verify-heading"
  >
    <h1 id="verify-heading" class="text-xl font-semibold tracking-tight">
      Confirm your email link
    </h1>
    {#if problem}<p class="mt-3 text-sm text-danger" role="alert">
        {problem}
      </p>{/if}
    {#if token}
      <p class="mt-2 text-sm text-text-muted">
        Continue to use the link you opened from your inbox.
      </p>
      <button
        type="button"
        disabled={busy}
        onclick={verify}
        class="mt-5 min-h-11 w-full rounded-lg bg-accent-candle px-4 py-2 text-sm font-semibold text-bg hover:bg-accent-candle-glow disabled:opacity-55"
      >
        {busy ? 'Checking your link…' : 'Continue'}
      </button>
    {/if}
    {#if ready}
      <a
        href={resolve('/login')}
        class="mt-2 inline-flex min-h-11 items-center text-sm font-medium text-accent-moss-strong underline underline-offset-4 hover:text-text"
      >
        Back to sign-in
      </a>
    {/if}
  </section>
</main>
