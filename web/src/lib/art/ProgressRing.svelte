<script lang="ts">
  // Animated progress donut: one coloured arc per segment, drawn in on mount.
  // Shared by the catalogue rail and the dashboard so progress reads the same everywhere.
  import { onMount } from 'svelte';

  type Segment = {
    /** How many items this segment represents. */
    value: number;
    /** CSS colour for the arc. */
    color: string;
  };

  let {
    segments = [],
    goal = 0,
    size = 132,
    stroke = 11,
    delayStep = 120
  }: {
    segments?: Segment[];
    /** Total items the ring counts toward; center shows the summed share. */
    goal?: number;
    size?: number;
    stroke?: number;
    /** Stagger between arcs, in ms. */
    delayStep?: number;
  } = $props();

  const radius = $derived((size - stroke) / 2);
  const circumference = $derived(2 * Math.PI * radius);
  const solved = $derived(segments.reduce((sum, s) => sum + s.value, 0));
  const spans = $derived.by(() => {
    let acc = 0;
    return segments.map((seg) => {
      const len = goal > 0 ? (seg.value / goal) * circumference : 0;
      const start = acc;
      acc += len;
      return { ...seg, len, start };
    });
  });

  let drawn = $state(false);
  onMount(() => {
    const id = requestAnimationFrame(() => (drawn = true));
    return () => cancelAnimationFrame(id);
  });
</script>

<div
  class="relative inline-grid place-items-center"
  style:width="{size}px"
  style:height="{size}px"
>
  <svg
    width={size}
    height={size}
    viewBox="0 0 {size} {size}"
    role="img"
    aria-label="{solved} of {goal} solved"
  >
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="var(--border)"
      stroke-width={stroke}
    />
    {#each spans as seg, i (i)}
      <circle
        cx={size / 2}
        cy={size / 2}
        r={radius}
        fill="none"
        stroke={seg.color}
        stroke-width={stroke}
        stroke-linecap="round"
        stroke-dasharray="{seg.len} {circumference - seg.len}"
        stroke-dashoffset={drawn ? -seg.start : -circumference}
        transform="rotate(-90 {size / 2} {size / 2})"
        style="transition: stroke-dashoffset 0.9s cubic-bezier(0.22, 1, 0.36, 1) {i *
          delayStep}ms"
      />
    {/each}
  </svg>
  <div class="absolute inset-0 grid place-items-center text-center">
    <div>
      <p class="text-2xl font-semibold tabular-nums leading-none">
        {solved}<span class="text-sm font-normal text-text-muted">/{goal}</span>
      </p>
      <p class="mt-1 text-[11px] tracking-wide text-text-muted uppercase">
        solved
      </p>
    </div>
  </div>
</div>
