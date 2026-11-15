<script lang="ts">
	import { HEADLINE, SUPPORT_LINE } from './copy';

	// Back to front. Each layer is a decorative image: no alt text, hidden from assistive technology.
	// Explicit sizes stop the layout from shifting while the SVGs load.
	const layers = [
		{ src: '/landing/moon.svg', width: 400, height: 400, name: 'moon' },
		{ src: '/landing/graveyard-far.svg', width: 1200, height: 240, name: 'far' },
		{ src: '/landing/smoke.svg', width: 800, height: 200, name: 'smoke' },
		{ src: '/landing/graveyard-near.svg', width: 1200, height: 220, name: 'near' },
		{ src: '/landing/bats.svg', width: 300, height: 120, name: 'bats' }
	];
</script>

<section class="relative isolate overflow-hidden px-6 pt-16 pb-10 sm:px-10">
	<div class="mx-auto flex max-w-3xl flex-col gap-6">
		<p class="font-mono text-sm tracking-[0.2em] text-accent-candle uppercase">Counterspell</p>
		<h1 class="text-[2.5rem] leading-tight font-semibold tracking-[-0.01em] text-text sm:text-[3.5rem]">
			{HEADLINE}
		</h1>
		<p class="max-w-xl text-lg leading-8 text-text-muted">{SUPPORT_LINE}</p>
	</div>

	<div class="scene relative mx-auto mt-10 h-56 max-w-3xl sm:h-72" aria-hidden="true">
		{#each layers as layer (layer.name)}
			<img
				class="layer layer-{layer.name} absolute inset-x-0 bottom-0 h-auto w-full"
				src={layer.src}
				alt=""
				aria-hidden="true"
				width={layer.width}
				height={layer.height}
				decoding="async"
			/>
		{/each}
	</div>
</section>
