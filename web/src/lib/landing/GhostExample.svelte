<script lang="ts">
  import { CAPTION, EXAMPLE } from './copy';

  // A fixed illustration of the Bug Autopsy shape. Nothing here is computed or fetched.
  // Every value has a text label, so colour is never the only signal.
</script>

<figure class="rounded border border-border bg-surface p-5 font-mono text-sm">
  <div class="space-y-2">
    <p class="text-text">
      <span class="text-text-subtle">input </span>{EXAMPLE.input}
    </p>
    <p class="text-text">
      <span class="text-text-subtle">expected </span><span class="text-success"
        >{EXAMPLE.expected}</span
      >
    </p>
    <p class="text-text">
      <span class="text-text-subtle">got </span><span class="text-danger"
        >{EXAMPLE.got}</span
      >
    </p>
  </div>
  <figcaption class="mt-3 text-xs text-text-subtle">{CAPTION}</figcaption>
</figure>
