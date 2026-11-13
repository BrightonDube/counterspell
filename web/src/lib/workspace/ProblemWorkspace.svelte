<script lang="ts">
  // Loads the problem and its languages for one slug. A new slug or a leave cancels the older request,
  // so a slow response can never replace the page the learner is looking at.
  import { loadLanguages } from '../api/languages';
  import { ApiRequestError } from '../api/client';
  import { loadProblem } from '../api/problems';
  import type { LanguageSummary, ProblemDetail } from '../api/types';
  import WorkspaceShell from './WorkspaceShell.svelte';

  let { slug }: { slug: string } = $props();

  type LoadState =
    | { kind: 'loading' }
    | { kind: 'ready'; problem: ProblemDetail; languages: LanguageSummary[] }
    | { kind: 'error'; message: string };

  let state = $state<LoadState>({ kind: 'loading' });

  $effect(() => {
    const requested = slug;
    const controller = new AbortController();
    const options = { signal: controller.signal };
    state = { kind: 'loading' };

    Promise.all([loadProblem(requested, options), loadLanguages(options)])
      .then(([problem, languages]) => {
        if (!controller.signal.aborted) {
          state = { kind: 'ready', problem, languages };
        }
      })
      .catch((error: unknown) => {
        if (controller.signal.aborted) return;
        // Only the server's own safe messages are shown. Anything else gets a fixed line.
        state = {
          kind: 'error',
          message:
            error instanceof ApiRequestError
              ? error.message
              : 'The workspace could not load. Try again later.'
        };
      });

    return () => controller.abort();
  });
</script>

{#if state.kind === 'loading'}
  <p role="status" class="text-sm text-text-muted">Loading problem…</p>
{:else if state.kind === 'error'}
  <p role="alert" class="text-sm text-danger">{state.message}</p>
{:else}
  <WorkspaceShell problem={state.problem} languages={state.languages} />
{/if}
