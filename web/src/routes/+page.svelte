<script lang="ts">
	import { onMount } from 'svelte';
	import { startScenes } from '../lib/landing/scenes';
	import { planLanding, type LandingPlan } from '../lib/landing/motion-plan';
	import GhostBeats from '../lib/landing/GhostBeats.svelte';
	import Hero from '../lib/landing/Hero.svelte';
	import SignUpCta from '../lib/landing/SignUpCta.svelte';
	// Vite fingerprints the font, so this URL matches the one the @font-face rule in app.css uses.
	import creepsterUrl from '../../static/fonts/Creepster-Regular.woff2?url';

	// The static HTML already shows every word. The plan only decides what motion to add.
	let plan = $state.raw<LandingPlan>(
		planLanding({ reducedMotion: false, viewportWidth: 1280, scriptsRunning: false })
	);

	onMount(() => {
		const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		const next = planLanding({
			reducedMotion,
			viewportWidth: window.innerWidth,
			scriptsRunning: true
		});
		plan = next;
		if (!next.loadGsap) return;

		// GSAP loads after a delay, so the page may be left first. Dispose whichever comes last.
		let active = true;
		let dispose: (() => void) | undefined;
		startScenes(next)
			.then((revert) => {
				if (active) dispose = revert;
				else revert();
			})
			.catch(() => {
				// Motion is optional. The static page already shows every word and beat.
			});
		return () => {
			active = false;
			dispose?.();
		};
	});
</script>

<svelte:head>
	<title>Counterspell</title>
	<link rel="preload" href={creepsterUrl} as="font" type="font/woff2" crossorigin="anonymous" />

	<meta
		name="description"
		content="Learn why your code breaks. Counterspell hunts the input that breaks your solution."
	/>
</svelte:head>

<main
	class="min-h-screen bg-bg text-text"
	lang="en"
	data-smoke-marker="counterspell-svelte-static"
	data-motion={plan.mode}
>
	<Hero {plan} />
	<div class="mx-auto flex max-w-3xl flex-col gap-16 px-6 py-16 sm:px-10">
		<div data-reveal>
			<GhostBeats />
		</div>
		<div data-reveal>
			<SignUpCta />
		</div>
	</div>
</main>
