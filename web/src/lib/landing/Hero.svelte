<script lang="ts">
	import { onMount } from 'svelte';
	import { HEADLINE, SUPPORT_LINE } from './copy';
	import { afterPaint } from './after-paint';
	import { chooseHeroVideo, type HeroVideo } from './hero-media';

	// The poster is the approved moon and graveyard still. It sits in the static HTML, so it is
	// the first thing painted and the page reads without any script.
	let video = $state<HeroVideo | null>(null);
	let videoEl = $state<HTMLVideoElement | undefined>(undefined);

	// The video is mounted only after first paint, and only when the visitor's settings allow it.
	onMount(() => {
		let active = true;
		afterPaint(() => {
			if (!active) return;
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
		<picture class="absolute inset-0 block h-full w-full">
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
			/>
		</picture>
		{#if video}
			<video
				bind:this={videoEl}
				class="absolute inset-0 h-full w-full object-cover"
				autoplay
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
