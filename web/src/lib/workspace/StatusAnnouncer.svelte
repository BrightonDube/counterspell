<script lang="ts">
  // A polite live region for short status text, such as the draft save state or a run
  // phase change. The message is plain text. srOnly keeps announcements off the page
  // visually while screen readers still read them.
  let {
    message = '',
    label,
    srOnly = false
  }: { message?: string; label?: string; srOnly?: boolean } = $props();
</script>

<p
  role="status"
  aria-live="polite"
  aria-label={label}
  class={srOnly ? 'sr-only' : 'text-sm text-text-muted'}
>
  {message}
</p>
