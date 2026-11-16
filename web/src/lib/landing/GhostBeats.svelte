<script lang="ts">
	import { BEATS } from './copy';
	import GhostExample from './GhostExample.svelte';

	// Each beat has one decorative picture from the approved art. The heading and text carry the
	// meaning, so the picture is hidden from assistive technology.
	const art = {
		Submit: '/art/objects/crypt-key.webp',
		Hunt: '/art/objects/crystal-ball.webp',
		Understand: '/art/objects/brain-jar.webp'
	} as const;
</script>

<div class="flex flex-col gap-12">
	{#each BEATS as beat (beat.heading)}
		<section class="flex flex-col gap-4">
			<div class="flex items-center gap-4">
				<img
					class="h-16 w-16 shrink-0 object-contain"
					src={art[beat.heading]}
					width="64"
					height="64"
					alt=""
					aria-hidden="true"
					loading="lazy"
					decoding="async"
				/>
				<h3 class="text-xl font-semibold tracking-[-0.01em] text-text">{beat.heading}</h3>
			</div>
			<p class="max-w-xl text-base leading-7 text-text-muted">{beat.body}</p>
			{#if beat.heading === 'Hunt'}
				<GhostExample />
			{/if}
		</section>
	{/each}
</div>
