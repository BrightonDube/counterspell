<script lang="ts">
  import { shortcutFor, type KeyInput } from './keyboard';
  import StatusAnnouncer from './StatusAnnouncer.svelte';

  let {
    busy,
    sourceBytes,
    limitBytes,
    saveMessage = '',
    onRun,
    onSubmit
  }: {
    busy: boolean;
    sourceBytes: number;
    limitBytes: number;
    saveMessage?: string;
    onRun: () => void;
    onSubmit: () => void;
  } = $props();

  // The byte check is advisory. The server repeats it and stays authoritative.
  const overLimit = $derived(sourceBytes > limitBytes);
  const blocked = $derived(busy || overLimit);

  // Form fields keep their own keys. The Monaco editor is a text area too, but its Ctrl+Enter is
  // treated as a run shortcut, as the design decided. Brighton can change this in ide-workspace-blockers.md.
  function isFormField(target: EventTarget | null): boolean {
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLSelectElement
    )
      return true;
    return (
      target instanceof HTMLTextAreaElement &&
      target.closest('.monaco-editor') === null
    );
  }

  function onKeydown(event: KeyboardEvent): void {
    const input: KeyInput = {
      key: event.key,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      shiftKey: event.shiftKey,
      inTextField: isFormField(event.target)
    };
    const shortcut = shortcutFor(input);
    if (shortcut === null || blocked) return;
    event.preventDefault();
    if (shortcut === 'run') onRun();
    else onSubmit();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="flex flex-col gap-2">
  <div class="flex flex-wrap gap-2">
    <button
      type="button"
      class="min-h-11 rounded border border-border-strong px-4 text-text disabled:opacity-60"
      disabled={blocked}
      aria-describedby="run-shortcuts"
      onclick={onRun}
    >
      Run samples
    </button>
    <button
      type="button"
      class="min-h-11 rounded bg-accent-candle px-4 font-semibold text-bg disabled:opacity-60"
      disabled={blocked}
      aria-describedby="run-shortcuts"
      onclick={onSubmit}
    >
      Submit
    </button>
  </div>
  <p id="run-shortcuts" class="text-sm text-text-muted">
    Ctrl+Enter or Cmd+Enter runs samples. Ctrl+Shift+Enter or Cmd+Shift+Enter
    submits.
  </p>
  {#if overLimit}
    <p role="alert" class="text-sm text-danger">
      This solution is {sourceBytes} bytes. The limit for this language is {limitBytes}
      bytes.
    </p>
  {/if}
  <StatusAnnouncer message={saveMessage} />
</div>
