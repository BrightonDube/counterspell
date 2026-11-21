<script lang="ts">
  // The animated jack-o'-lantern familiar: an AnimatedArt specialization for the
  // mood webm pair shipped under /art/familiar (orange = steady candle, poison =
  // the green-lit variant used for the newest accepted run).
  import AnimatedArt from '../art/AnimatedArt.svelte';

  let {
    mood = 'orange',
    size = 96,
    centre = true
  }: { mood?: 'orange' | 'poison'; size?: number; centre?: boolean } = $props();
</script>

<AnimatedArt
  src="/art/familiar/jack-o-lantern-{mood}.webp"
  srcset="/art/familiar/jack-o-lantern-{mood}-192.webp 192w, /art/familiar/jack-o-lantern-{mood}.webp 512w"
  sizes="96px"
  webm="/art/familiar/jack-o-lantern-{mood}.webm"
  {size}
  motion="breathe"
  bounce
  {centre}
/>
