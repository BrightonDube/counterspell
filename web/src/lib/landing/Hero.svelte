<script lang="ts">
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { TAGLINE, WORDMARK } from './copy';
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

	// The ENTER link. Pressing Enter on the title screen follows it, but only when no other control has focus.
	let enterLink = $state<HTMLAnchorElement | undefined>(undefined);

	onMount(() => {
		function onEnterKey(event: KeyboardEvent) {
			if (event.key !== 'Enter' || event.repeat || event.defaultPrevented) return;
			const focused = document.activeElement;
			if (focused && focused !== document.body && focused !== document.documentElement) return;
			// Only on the title screen: once the visitor has scrolled past it, Enter does nothing special.
			if (window.scrollY > window.innerHeight * 0.5) return;
			event.preventDefault();
			enterLink?.click();
		}
		window.addEventListener('keydown', onEnterKey);
		return () => window.removeEventListener('keydown', onEnterKey);
	});

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
	class="hero relative isolate flex flex-col items-center overflow-hidden px-6 pt-[12vh] pb-[20vh] sm:px-10"
>
	<div class="hero-media absolute inset-0 -z-10 overflow-hidden" aria-hidden="true">
		<div class="hero-plate">
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
			<picture class="absolute inset-0 block h-full w-full">
				<source
					type="image/webp"
					srcset="/art/landing/hero-moon-living-poster-960.webp 960w, /art/landing/hero-moon-living-poster-1920.webp 1920w"
					sizes="(max-width: 900px) 100vw, 1920px"
				/>
				<img
					class="hero-poster h-full w-full object-cover"
					src="/art/landing/hero-moon-living-poster.jpg"
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

		<div class="pumpkin-glow" aria-hidden="true"></div>
		</div>
		<div class="hero-scrim absolute inset-0"></div>
	</div>

	<div class="hero-brand relative mx-auto flex w-full max-w-3xl flex-col items-center gap-4 text-center">
		<h1 class="wordmark">{WORDMARK}</h1>
		<p class="tagline">{TAGLINE}</p>
	</div>

	<a
		bind:this={enterLink}
		class="enter-link"
		href={resolve('/login')}
		aria-label="Enter, go to sign in"
	>
		<span class="enter-label" aria-hidden="true">ENTER</span>
		<span class="enter-hint" aria-hidden="true">Press to begin</span>
	</a>
</section>

<style>
	/* Full viewport, with a fallback for browsers without the small-viewport unit. */
	.hero {
		min-height: 100vh;
		min-height: 100svh;
	}

	.hero-media {
		container-type: size;
	}

	/* The painting is a 16:9 plate that covers the hero, so the pumpkin glow lines up with the
	   painted pumpkin at every size (the same geometry as the approved mockup). */
	.hero-plate {
		position: absolute;
		top: 50%;
		left: 50%;
		width: max(100%, calc(100cqh * 16 / 9));
		height: max(100%, calc(100cqw * 9 / 16));
		transform: translate(-50%, -50%);
	}

	/* On phones the plate is shifted so the painted pumpkin stays on screen at about a fifth of the width. */
	@media (max-width: 700px) {
		.hero-plate {
			left: 0;
			transform: translate(calc(-14.1% + 20vw), -50%);
		}
	}

	.pumpkin-glow {
		position: absolute;
		left: 14.1%;
		top: 67.2%;
		width: 26%;
		height: 40%;
		z-index: 2;
		pointer-events: none;
		transform: translate(-50%, -50%);
		background: radial-gradient(
			ellipse at 52% 48%,
			rgb(var(--hero-ember) / 0.4) 0%,
			rgb(var(--hero-ember) / 0.18) 30%,
			rgb(var(--hero-ember) / 0.07) 52%,
			transparent 70%
		);
		mix-blend-mode: screen;
		filter: blur(1.5px);
		animation: ember-pulse 2.8s ease-in-out infinite;
	}

	/* Wordmark and tagline: real text in Creepster and Geist, lit by the same ember colour as the pumpkin. */
	.wordmark {
		font-family: var(--font-wordmark);
		font-size: clamp(2.4rem, 9.2vw, 7.2rem);
		font-weight: 400;
		line-height: 0.95;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--color-text);
		text-shadow:
			0 0 8px color-mix(in oklab, var(--bg) 95%, transparent),
			0 2px 8px color-mix(in oklab, var(--bg) 95%, transparent),
			0 0 16px rgb(var(--hero-ember) / 0.55),
			0 0 42px rgb(var(--hero-ember) / 0.45),
			0 0 70px rgb(var(--hero-ember) / 0.2);
	}

	.tagline {
		font-weight: 500;
		font-size: clamp(0.8rem, 1.35vw, 1.125rem);
		letter-spacing: 0.28em;
		text-indent: 0.28em;
		text-transform: uppercase;
		color: var(--color-text);
		text-shadow:
			0 0 8px rgb(var(--hero-ember) / 0.55),
			0 0 22px rgb(var(--hero-ember) / 0.35),
			0 2px 10px color-mix(in oklab, var(--bg) 95%, transparent);
	}

	/* A soft dark pool behind the wordmark keeps the bone letters legible over the bright moon. */
	.hero-brand::before {
		content: '';
		position: absolute;
		left: 50%;
		top: 44%;
		width: 118%;
		height: 165%;
		transform: translate(-50%, -50%);
		background: radial-gradient(
			ellipse at 50% 48%,
			color-mix(in oklab, var(--bg) 52%, transparent) 0%,
			color-mix(in oklab, var(--bg) 18%, transparent) 48%,
			transparent 72%
		);
		z-index: -1;
		pointer-events: none;
	}

	/* ENTER: bottom centre, Creepster label, pulsing glow. The pulse changes only shadow and brightness, never
	   position, so it never shifts the link under the pointer. */
	.enter-link {
		position: absolute;
		bottom: 6vh;
		left: 50%;
		z-index: 2;
		transform: translateX(-50%);
		display: inline-flex;
		flex-direction: column;
		align-items: center;
		padding: 1.05rem 3.2rem 1.15rem;
		border-radius: 6px;
		border: 1px solid rgb(var(--hero-ember) / 0.8);
		background: color-mix(in oklab, var(--bg) 55%, transparent);
		box-shadow:
			0 0 24px rgb(var(--hero-ember) / 0.5),
			0 0 60px rgb(var(--hero-ember) / 0.28);
		animation: enter-pulse 2.2s ease-in-out infinite;
	}

	.enter-link:focus-visible {
		outline: 2px solid var(--focus);
		outline-offset: 5px;
	}

	.enter-label {
		font-family: var(--font-wordmark);
		font-size: clamp(2rem, 3.6vw, 3rem);
		line-height: 1;
		letter-spacing: 0.2em;
		text-indent: 0.2em;
		text-transform: uppercase;
		color: var(--accent-candle-glow);
		text-shadow:
			0 0 14px rgb(var(--hero-ember) / 0.95),
			0 0 32px rgb(var(--hero-ember) / 0.7);
	}

	.enter-hint {
		margin-top: 0.48rem;
		font-weight: 600;
		font-size: 0.75rem;
		letter-spacing: 0.32em;
		text-indent: 0.32em;
		text-transform: uppercase;
		color: var(--color-text);
	}

	@keyframes enter-pulse {
		0%,
		100% {
			filter: brightness(1);
		}
		50% {
			filter: brightness(1.22);
		}
	}

	@keyframes ember-pulse {
		0%,
		100% {
			opacity: 0.72;
			transform: translate(-50%, -50%) scale(1);
		}
		50% {
			opacity: 1;
			transform: translate(-50%, -50%) scale(1.07);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.pumpkin-glow,
		.enter-link {
			animation: none;
		}
	}
</style>
