<script lang="ts">
	import { onMount } from 'svelte';
	import { HEADLINE, SUPPORT_LINE } from './copy';
	import { afterPaint } from './after-paint';
	import { chooseHeroVideo, type HeroVideo } from './hero-media';
	import { planLanding, type LandingPlan } from './motion-plan';

	// The plan decides which motion overlays exist. The static default matches a page with no script.
	let { plan = planLanding({ reducedMotion: false, viewportWidth: 1280, scriptsRunning: false }) }: {
		plan?: LandingPlan;
	} = $props();

	// The poster is the approved moon and graveyard still. It sits in the static HTML, so it is
	// the first thing painted and the page reads without any script.
	let video = $state<HeroVideo | null>(null);
	let videoEl = $state<HTMLVideoElement | undefined>(undefined);
	// If the poster cannot load, the flat placeholder SVGs stand in for it.
	let posterFailed = $state(false);
	// Overlays wait for first paint, so the poster is the only image competing for the first bytes.
	let overlaysReady = $state(false);

	// The video is mounted only after first paint, and only when the visitor's settings allow it.
	onMount(() => {
		let active = true;
		afterPaint(() => {
			if (!active) return;
			overlaysReady = true;
			const connection = (navigator as Navigator & { connection?: { saveData?: boolean } })
				.connection;
			video = chooseHeroVideo({
				reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
				viewportWidth: window.innerWidth,
				saveData: connection?.saveData === true,
				tabVisible: !document.hidden
			});
		});
		return () => {
			active = false;
		};
	});

	// Play only while the hero is on screen and the tab is visible. Pause otherwise.
	$effect(() => {
		const el = videoEl;
		if (!el) return;
		const play = () => void el.play().catch(() => {});
		const observer = new IntersectionObserver(([entry]) => {
			if (entry.isIntersecting && !document.hidden) play();
			else el.pause();
		});
		observer.observe(el);
		const onVisibility = () => (document.hidden ? el.pause() : play());
		document.addEventListener('visibilitychange', onVisibility);
		return () => {
			observer.disconnect();
			document.removeEventListener('visibilitychange', onVisibility);
		};
	});
</script>

<section
	class="relative isolate flex min-h-[32rem] items-end overflow-hidden px-6 pt-24 pb-16 sm:px-10"
>
	<div class="absolute inset-0 -z-10" aria-hidden="true">
		{#if posterFailed}
			<div class="layer-moon absolute inset-0">
				<img
					class="absolute top-[10%] right-[12%] h-[40%] w-auto"
					src="/landing/moon.svg"
					width="400"
					height="400"
					alt=""
					aria-hidden="true"
				/>
				<img
					class="layer-far absolute inset-x-0 bottom-0 h-auto w-full"
					src="/landing/graveyard-far.svg"
					width="1200"
					height="240"
					alt=""
					aria-hidden="true"
				/>
				<img
					class="layer-near absolute inset-x-0 bottom-0 h-auto w-full"
					src="/landing/graveyard-near.svg"
					width="1200"
					height="220"
					alt=""
					aria-hidden="true"
				/>
			</div>
		{:else}
			<picture class="layer-moon absolute inset-0 block h-full w-full">
				<source
					type="image/webp"
					srcset="/art/landing/moon-graveyard-poster-960.webp 960w, /art/landing/moon-graveyard-poster-1920.webp 1920w"
					sizes="(max-width: 900px) 100vw, 1920px"
				/>
				<img
					class="h-full w-full object-cover"
					src="/art/landing/moon-graveyard-poster-1920.jpg"
					width="1920"
					height="1080"
					alt=""
					aria-hidden="true"
					fetchpriority="high"
					decoding="async"
					onerror={() => (posterFailed = true)}
				/>
			</picture>
		{/if}

		{#if video}
			<video
				bind:this={videoEl}
				class="absolute inset-0 h-full w-full object-cover"
				muted
				loop
				playsinline
				preload="none"
				disablepictureinpicture
			>
				<source src={video.webm} type="video/webm" />
				<source src={video.mp4} type="video/mp4" />
			</video>
		{/if}

		{#if overlaysReady && plan.layers.far}
			<img
				class="layer-far absolute inset-x-0 bottom-0 h-auto w-full"
				src="/art/fx/fog-layer-cold.webp"
				width="1600"
				height="400"
				alt=""
				aria-hidden="true"
				decoding="async"
			/>
		{/if}

		{#if overlaysReady && plan.layers.near}
			<img
				class="layer-near absolute bottom-0 left-0 h-auto w-2/3"
				src="/art/fx/embers.webp"
				width="400"
				height="300"
				alt=""
				aria-hidden="true"
				decoding="async"
			/>
		{/if}

		{#if overlaysReady && plan.layers.smoke}
			<!-- Starts at opacity 0 so it never pops in before the scene timelines set it. -->
			<img
				class="layer-smoke absolute right-[10%] bottom-0 h-[70%] w-auto"
				style="opacity: 0"
				src="/art/fx/smoke-plume-ash.webp"
				width="300"
				height="600"
				alt=""
				aria-hidden="true"
				decoding="async"
			/>
		{/if}

		{#if overlaysReady && plan.bats}
			<!-- Starts at opacity 0 so the flock is not visible before the one-shot flight begins. -->
			<img
				class="layer-bats absolute top-[18%] left-0 h-auto w-1/2"
				style="opacity: 0"
				src="/art/fx/bat-swarm.webp"
				width="1600"
				height="500"
				alt=""
				aria-hidden="true"
				decoding="async"
			/>
		{/if}

		<div class="hero-scrim absolute inset-0"></div>
	</div>

	<div class="relative mx-auto flex w-full max-w-3xl flex-col gap-6">
		<p class="font-mono text-sm tracking-[0.2em] text-accent-candle uppercase">Counterspell</p>
		<h1 class="text-[2.5rem] leading-tight font-semibold tracking-[-0.01em] text-text sm:text-[3.5rem]">
			{HEADLINE}
		</h1>
		<p class="max-w-xl text-lg leading-8 text-text-muted">{SUPPORT_LINE}</p>
	</div>
</section>
