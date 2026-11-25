<script lang="ts">
  import { goto } from '$app/navigation';
  import { browser } from '$app/env';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import ProfileForm from '#lib/auth/ProfileForm.svelte';
  import { postAuthTarget, returnToFromQuery } from '#lib/auth/return-to.js';
  import { session } from '#lib/auth/session.svelte.js';
  import Seo from '#lib/seo/Seo.svelte';

  let loading = $state(true);
  let problem = $state('');
  const destination = $derived(
    postAuthTarget(returnToFromQuery(browser ? page.url.search : ''))
  );

  onMount(async () => {
    try {
      await session.load();
      if (session.status === 'signed_out') {
        await goto(
          resolve(
            `/login?return_to=${encodeURIComponent(`/onboarding${page.url.search}`)}`
          )
        );
        return;
      }
      if (session.user?.is_demo) {
        await goto(resolve('/'));
        return;
      }
    } catch {
      problem = 'Your profile could not be loaded. Try again later.';
    } finally {
      loading = false;
    }
  });
</script>

<Seo
  title="Set up your profile · Counterspell"
  description="Choose the name Counterspell shows on your progress and runs."
  path="/onboarding"
  social={false}
/>

<main class="onboarding">
  <section class="card">
    {#if loading}
      <p role="status">Loading your profile…</p>
    {:else if problem}
      <p role="alert">{problem}</p>
    {:else if session.user}
      <img
        class="avatar"
        src={session.user.avatar_url || '/art/creatures/skull-green-eyes.webp'}
        width="96"
        height="96"
        alt=""
      />
      <h1>Set up your profile</h1>
      <p class="intro">
        Tell Counterspell what to call you — the name shown on your progress and
        runs. The skull keeps watch as your avatar.
      </p>
      <ProfileForm
        submitLabel="Continue"
        onSaved={() => void goto(resolve(destination as '/'))}
      />
    {/if}
  </section>
</main>

<style>
  .onboarding {
    display: grid;
    min-block-size: 100dvh;
    place-items: center;
    padding: 2rem 1rem;
  }

  .card {
    display: grid;
    inline-size: min(100%, 28rem);
    gap: 1rem;
    padding: clamp(1.25rem, 4vw, 2rem);
    border: 1px solid var(--border-strong);
    border-radius: 0.75rem;
    background: var(--surface);
  }

  .avatar {
    inline-size: 6rem;
    block-size: 6rem;
    border-radius: 50%;
    filter: drop-shadow(
      0 0 0.7rem color-mix(in srgb, var(--accent-candle) 45%, transparent)
    );
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    letter-spacing: -0.03em;
  }

  .intro {
    margin: 0;
    color: var(--text-muted);
  }
</style>
