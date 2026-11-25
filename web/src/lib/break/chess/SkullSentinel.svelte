<script>
  // The skull opponent (CH06): decorative atmosphere above the board. Lazy,
  // low-priority, aria-hidden — it never touches move input, and the global
  // reduced-motion collapse stills its hover.
  /** @type {{ src?: string }} */
  let { src = '/art/creatures/skull-green-eyes.webp' } = $props();
</script>

<img
  class="skull-sentinel"
  {src}
  alt=""
  aria-hidden="true"
  loading="lazy"
  decoding="async"
  fetchpriority="low"
  width="96"
  height="96"
/>
