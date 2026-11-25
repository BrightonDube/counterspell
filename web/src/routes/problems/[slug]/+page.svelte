<script lang="ts">
  import { page } from '$app/state';
  import Seo from '../../../lib/seo/Seo.svelte';
  import ProblemWorkspace from '../../../lib/workspace/ProblemWorkspace.svelte';
  import type { ProblemDetail } from '../../../lib/api/types';

  const slug = $derived(page.params.slug ?? '');
  let problem = $state<ProblemDetail | undefined>();

  // Navigating to another slug must drop the previous problem — otherwise the
  // title/description keep naming it while the next one loads.
  $effect(() => {
    if (slug) problem = undefined;
  });

  const title = $derived(
    problem
      ? `${problem.title} · Counterspell`
      : 'Problem workspace · Counterspell'
  );
  const description = $derived(
    problem
      ? `${problem.title} — a ${problem.difficulty} ${problem.track.name} problem in Counterspell.`
      : 'Solve a Counterspell problem with instant judge feedback.'
  );
</script>

<Seo {title} {description} path="/problems/{slug}" />

<main class="min-h-screen bg-bg px-4 py-6 text-text sm:px-6">
  <ProblemWorkspace {slug} onready={(p) => (problem = p)} />
</main>
