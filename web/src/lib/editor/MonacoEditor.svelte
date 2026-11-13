<script lang="ts">
  // Lazy Monaco editor boundary. It creates the editor after Monaco loads, keeps one model per language
  // so drafts survive a language switch, reports source changes, and disposes everything on unmount.
  // It never submits a run; the workspace decides what to do with the source.
  import { untrack } from 'svelte';
  import { SvelteMap } from 'svelte/reactivity';
  import type * as Monaco from 'monaco-editor';
  import { CRYPT_THEME_NAME, loadMonaco } from './monaco-loader';

  let {
    value,
    language,
    label,
    onChange
  }: {
    value: string;
    language: string;
    label: string;
    onChange: (source: string) => void;
  } = $props();

  let host: HTMLDivElement | undefined = $state();
  let status = $state<'loading' | 'ready' | 'failed'>('loading');
  let attempt = $state(0);

  // Monaco objects are plain handles, not Svelte state, so they are kept outside the reactive graph.
  let api: typeof Monaco | null = null;
  let editor: Monaco.editor.IStandaloneCodeEditor | null = null;
  const models = new SvelteMap<string, Monaco.editor.ITextModel>();

  // Creates the editor once per attempt. Props are read untracked: later language changes are handled below.
  $effect(() => {
    const container = host;
    void attempt;
    if (!container) return;

    let cancelled = false;
    status = 'loading';
    loadMonaco()
      .then((monaco) => {
        if (cancelled) return;
        api = monaco;
        editor = monaco.editor.create(container, {
          theme: CRYPT_THEME_NAME,
          ariaLabel: untrack(() => label),
          automaticLayout: true,
          minimap: { enabled: false }
        });
        editor.onDidChangeModelContent(() => {
          onChange(editor?.getValue() ?? '');
        });
        status = 'ready';
      })
      .catch(() => {
        if (!cancelled) status = 'failed';
      });

    return () => {
      cancelled = true;
      editor?.dispose();
      for (const model of models.values()) model.dispose();
      models.clear();
      editor = null;
      api = null;
    };
  });

  // Shows the model for the current language. A language seen before keeps its own content.
  $effect(() => {
    const current = language;
    if (status !== 'ready' || !api || !editor) return;
    let model = models.get(current);
    if (!model) {
      model = api.editor.createModel(
        untrack(() => value),
        current
      );
      models.set(current, model);
    }
    editor.setModel(model);
  });
</script>

<div class="flex flex-col gap-2">
  {#if status === 'loading'}
    <p role="status" class="text-sm text-text-muted">Loading editor</p>
  {:else if status === 'failed'}
    <div class="flex flex-col items-start gap-2">
      <p role="alert" class="text-sm text-danger">
        The editor could not load. Your source is still in the page.
      </p>
      <button
        type="button"
        class="min-h-11 rounded border border-border-strong px-4 text-text"
        onclick={() => (attempt += 1)}
      >
        Try again
      </button>
    </div>
  {/if}
  <div bind:this={host} class="min-h-64 flex-1"></div>
</div>
