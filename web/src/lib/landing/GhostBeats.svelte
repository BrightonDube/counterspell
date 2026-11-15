<script lang="ts">
	import { BEATS } from './copy';
	import GhostExample from './GhostExample.svelte';

	// Each beat has a decorative icon. The heading and text carry the meaning, so the icon is hidden.
	const icons = {
		Submit:
			'M4 6h16v12H4z M8 10l3 3 5-5',
		Hunt:
			'M11 4a7 7 0 1 0 4.9 12l4.1 4.1 1.4-1.4-4.1-4.1A7 7 0 0 0 11 4z',
		Understand:
			'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z M12 8v5 M12 16h.01'
	} as const;
</script>

<div class="flex flex-col gap-12">
	{#each BEATS as beat (beat.heading)}
		<section class="flex flex-col gap-4">
			<svg
				class="h-8 w-8 text-accent-moss"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
				focusable="false"
			>
				<path d={icons[beat.heading]} />
			</svg>
			<h2 class="text-2xl font-semibold tracking-[-0.01em] text-text">{beat.heading}</h2>
			<p class="max-w-xl text-base leading-7 text-text-muted">{beat.body}</p>
			{#if beat.heading === 'Hunt'}
				<GhostExample />
			{/if}
		</section>
	{/each}
</div>
