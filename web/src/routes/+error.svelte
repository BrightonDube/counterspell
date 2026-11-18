<script lang="ts">
  import { page } from '$app/state';
  import ErrorView from '../lib/errors/ErrorView.svelte';
</script>

<svelte:head>
  <title>{page.status} · Counterspell</title>
</svelte:head>

<main class="min-h-screen bg-bg text-text" lang="en">
  <ErrorView status={page.status} />
</main>
