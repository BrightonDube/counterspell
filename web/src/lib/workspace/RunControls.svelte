<script lang="ts">
  let {
    busy,
    sourceBytes,
    limitBytes,
    onRun,
    onSubmit
  }: {
    busy: boolean;
    sourceBytes: number;
    limitBytes: number;
    onRun: () => void;
    onSubmit: () => void;
  } = $props();

  // The byte check is advisory. The server repeats it and stays authoritative.
  const overLimit = $derived(sourceBytes > limitBytes);
  const blocked = $derived(busy || overLimit);
</script>

<div class="flex flex-col gap-2">
  <div class="flex flex-wrap gap-2">
    <button
      type="button"
      class="min-h-11 rounded border border-border-strong px-4 text-text disabled:opacity-60"
      disabled={blocked}
      onclick={onRun}
    >
      Run samples
    </button>
    <button
      type="button"
      class="min-h-11 rounded bg-accent-candle px-4 font-semibold text-bg disabled:opacity-60"
      disabled={blocked}
      onclick={onSubmit}
    >
      Submit
    </button>
  </div>
  {#if overLimit}
    <p role="alert" class="text-sm text-danger">
      This solution is {sourceBytes} bytes. The limit for this language is {limitBytes}
      bytes.
    </p>
  {/if}
</div>
