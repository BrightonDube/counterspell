<script lang="ts">
  import { page } from '$app/state';
  import ErrorView from '../lib/errors/ErrorView.svelte';
  import Seo from '../lib/seo/Seo.svelte';
</script>

<Seo
  title="{page.status} · Counterspell"
  description="This Counterspell page could not be shown."
  social={false}
/>

<main class="min-h-screen bg-bg text-text" lang="en">
  <ErrorView status={page.status} />
</main>
