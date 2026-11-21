<script lang="ts">
  let {
    name
  }: {
    name:
      | 'key'
      | 'shield'
      | 'lock'
      | 'sign-out'
      | 'trash'
      | 'plus'
      | 'chevron'
      | 'check';
  } = $props();
</script>

<svg
  class="size-5 shrink-0"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.65"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'key'}
    <path d="m15.5 14.5-7 7H5v-4l2-2h2v-2l2.4-2.4a6 6 0 1 1 4.1 3.4Z" />
    <circle cx="16.5" cy="8" r="1" />
  {:else if name === 'shield'}
    <path d="M12 22s8-3.6 8-10V5l-8-3-8 3v7c0 6.4 8 10 8 10Z" />
    <path d="m9 12 2 2 4-4" />
  {:else if name === 'lock'}
    <rect x="5" y="10" width="14" height="11" rx="2" />
    <path d="M8 10V7a4 4 0 0 1 8 0v3m-4 4v3" />
  {:else if name === 'sign-out'}
    <path d="M11 4H5v16h6m-1-8h11m-4-4 4 4-4 4" />
  {:else if name === 'trash'}
    <path d="M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7m4-7v7" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'chevron'}
    <path d="m9 18 6-6-6-6" />
  {:else}
    <path d="m5 12 4 4L19 6" />
  {/if}
</svg>
