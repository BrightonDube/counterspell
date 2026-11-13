<script lang="ts">
  // A polite live region for short status text, such as the draft save state. The message is plain text.
  let { message = '' }: { message?: string } = $props();
</script>

<p role="status" aria-live="polite" class="text-sm text-text-muted">
  {message}
</p>
