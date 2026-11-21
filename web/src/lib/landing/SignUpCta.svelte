<script lang="ts">
  import { resolve } from '$app/paths';
  import { CTA } from './copy';
  import AnimatedArt from '../art/AnimatedArt.svelte';
  import AnimatedJacko from '../familiar/AnimatedJacko.svelte';

  // One call to action: 'Start practising' opens the grimoire. Sign-in lives at ENTER above.
  // The candle is decorative: it is hidden from assistive technology and sized so the layout does not shift.
</script>

<div class="flex flex-wrap items-center gap-6">
  <AnimatedArt
    src="/art/objects/candle-flame.webp"
    srcset="/art/objects/candle-flame-192.webp 192w, /art/objects/candle-flame.webp 512w"
    size={96}
    sizes="96px"
    motion="flame"
    bounce
  />
  <a
    href={resolve('/problems')}
    class="cta inline-flex items-center rounded bg-accent-candle px-6 py-3 text-base font-semibold text-bg transition-colors duration-200 hover:bg-accent-candle-glow focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-bg focus-visible:outline-none"
  >
    {CTA.primary}
  </a>
  <AnimatedJacko mood="orange" size={96} />
</div>
