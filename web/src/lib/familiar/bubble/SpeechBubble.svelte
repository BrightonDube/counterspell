<script lang="ts">
  import type { SpeechResult } from '../../api/familiar';

  let { speech }: { speech: SpeechResult } = $props();
</script>

<!-- Text only: Svelte escapes the body, so nothing a model writes can run as HTML. -->
<div role="status" class="w-full rounded bg-bg px-3 py-2 text-sm text-text">
  {#if speech.kind === 'ready'}
    {#if speech.label === 'AI hint'}
      <span class="font-semibold">AI hint:</span>
    {/if}
    {speech.text}
    {#if speech.label === 'AI hint'}
      {#if speech.sentToProvider}
        <p class="mt-1 text-xs text-text-muted">
          Your code was sent to a third-party AI provider to write this.
        </p>
      {/if}
    {:else}
      <p class="mt-1 text-xs text-text-muted">From your measured results</p>
    {/if}
  {:else}
    {speech.text}
  {/if}
</div>
