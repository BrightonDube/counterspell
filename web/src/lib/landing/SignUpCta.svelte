<script lang="ts">
  import { resolve } from '$app/paths';
  import { CTA } from './copy';
  import AnimatedArt from '../art/AnimatedArt.svelte';
  import AnimatedJacko from '../familiar/AnimatedJacko.svelte';

  // 'Start practising' opens the grimoire; 'Log in' goes to sign-in (GitHub or Google).
  // The candle is decorative: it is hidden from assistive technology and sized so the layout does not shift.
</script>

<div class="flex flex-wrap items-center gap-6">
  <a
    href={resolve('/problems')}
    class="cta inline-flex items-center rounded bg-accent-candle px-6 py-3 text-base font-semibold text-bg transition-colors duration-200 hover:bg-accent-candle-glow focus-visible:ring-2 focus-visible:ring-focus focus-visible:ring-offset-2 focus-visible:ring-offset-bg focus-visible:outline-none"
  >
    {CTA.primary}
  </a>
  <a
    href={resolve('/login')}
    class="text-base text-accent-moss underline-offset-4 hover:text-accent-moss-strong hover:underline focus-visible:ring-2 focus-visible:ring-focus focus-visible:outline-none"
  >
    {CTA.secondary}
  </a>
  <AnimatedArt
    src="/art/objects/candle-flame.webp"
    srcset="/art/objects/candle-flame-192.webp 192w, /art/objects/candle-flame.webp 512w"
    size={96}
    sizes="96px"
    motion="flame"
    bounce
  />
  <AnimatedJacko mood="orange" size={96} />
</div>
