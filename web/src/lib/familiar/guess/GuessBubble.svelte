<script lang="ts">
  // Live-typing guess bubble (efficiency-familiar T48, R14.1/R14.6).
  //
  // Off by default. The first enable spawns the Web Worker, which lazily
  // fetches the tree-sitter runtime and grammar .wasm files; nothing loads at
  // page load. The guess is local only — never sent to the server and never
  // an input to the mood path.
  import type { Guess } from './heuristics';

  let {
    source,
    language
  }: {
    /** Current editor text; re-guessed after a ~700 ms pause. */
    source: string;
    /** Language id from the workspace catalogue ('python', 'javascript', ...). */
    language: string;
  } = $props();

  const DEBOUNCE_MS = 700;

  let enabled = $state(false);
  let guess = $state<Guess | null>(null);
  let worker: Worker | null = null;
  let seq = 0;

  // Only the grammars budgeted in T46 support a guess; the switch hides
  // entirely for languages without one.
  const supported = $derived(
    language === 'python' ||
      language === 'javascript' ||
      language === 'typescript'
  );

  $effect(() => {
    if (!enabled || !supported) return;
    worker ??= new Worker(new URL('./worker.ts', import.meta.url), {
      type: 'module'
    });
    const current = worker;
    current.onmessage = (
      event: MessageEvent<{ seq: number; guess: Guess | null }>
    ) => {
      if (event.data.seq === seq) guess = event.data.guess;
    };
    const timer = setTimeout(() => {
      seq += 1;
      current.postMessage({ seq, code: source, language });
    }, DEBOUNCE_MS);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    return () => {
      worker?.terminate();
      worker = null;
    };
  });
</script>

{#if supported}
  <div class="flex flex-col gap-2 rounded border border-border bg-surface p-3">
    <label class="flex min-h-11 items-center gap-2 text-sm text-text">
      <input
        type="checkbox"
        bind:checked={enabled}
        aria-label="Live complexity guess"
      />
      Live complexity guess
    </label>
    {#if enabled}
      <p aria-live="polite" class="text-sm text-text">
        {#if guess?.guess_class}
          Looks {guess.guess_class} ({guess.confidence} confidence).
        {:else}
          Reading your code…
        {/if}
      </p>
      <p class="text-xs text-text-muted">
        Guess from reading your code. Run it for the real verdict.
      </p>
    {/if}
  </div>
{/if}
