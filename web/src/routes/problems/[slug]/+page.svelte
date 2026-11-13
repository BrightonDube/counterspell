<script lang="ts">
  import { page } from '$app/state';
  import ProblemWorkspace from '../../../lib/workspace/ProblemWorkspace.svelte';
</script>

<svelte:head>
  <title>Problem workspace · Counterspell</title>
</svelte:head>

<main class="min-h-screen bg-bg px-4 py-6 text-text sm:px-6">
  <ProblemWorkspace slug={page.params.slug ?? ''} />
</main>
