// GSAP scene timelines for the landing page. This is the only module that imports GSAP.
// It loads after first paint and only when the motion plan asks for it.
// Every animation plays once, uses transform and opacity only, and is reverted on dispose.

import { afterPaint } from './after-paint';
import type { LandingPlan } from './motion-plan';

/** The parts of GSAP this module uses. Typed narrowly so tests can pass a stand-in. */
interface GsapLike {
	registerPlugin: (...plugins: unknown[]) => void;
	context: (fn: () => void, scope?: ParentNode) => { revert: () => void };
	from: (targets: unknown, vars: Record<string, unknown>) => unknown;
	fromTo: (targets: unknown, from: Record<string, unknown>, to: Record<string, unknown>) => unknown;
	to: (targets: unknown, vars: Record<string, unknown>) => unknown;
}

export interface SceneDeps {
	/** Runs the callback once the browser is idle, after first paint. */
	schedule: (callback: () => void) => void;
	/** Loads GSAP and its ScrollTrigger plugin. The real module uses a dynamic import. */
	load: () => Promise<{ gsap: GsapLike; ScrollTrigger: unknown }>;
}

const defaultLoad: SceneDeps['load'] = async () => {
	const [{ gsap }, { ScrollTrigger }] = await Promise.all([
		import('gsap'),
		import('gsap/ScrollTrigger')
	]);
	return { gsap: gsap as unknown as GsapLike, ScrollTrigger };
};

const DEFAULT_DEPS: SceneDeps = { schedule: afterPaint, load: defaultLoad };

/**
 * Starts the landing scenes for a motion plan and returns a function that reverts them.
 *
 * Static plans return a no-op dispose and never load GSAP. Motion plans load GSAP after first paint,
 * then set up reveals, the moon, the parallax, the smoke and the one-shot bat flight under `root`.
 */
export function startScenes(
	plan: LandingPlan,
	root: Element = document.body,
	deps: SceneDeps = DEFAULT_DEPS
): Promise<() => void> {
	if (!plan.loadGsap) {
		return Promise.resolve(() => {});
	}

	return new Promise((resolve, reject) => {
		deps.schedule(() => {
			deps
				.load()
				.then(({ gsap, ScrollTrigger }) => {
					gsap.registerPlugin(ScrollTrigger);
					const context = gsap.context(() => buildTimelines(gsap, plan, root), root);
					resolve(() => context.revert());
				})
				.catch(reject);
		});
	});
}

/** Adds the tweens for each part of the plan. Called inside a GSAP context so they can be reverted together. */
function buildTimelines(gsap: GsapLike, plan: LandingPlan, root: Element): void {
	if (plan.reveals) {
		for (const reveal of Array.from(root.querySelectorAll('[data-reveal]'))) {
			gsap.from(reveal, {
				opacity: 0,
				y: 16,
				duration: 0.8,
				ease: 'power2.out',
				immediateRender: false,
				scrollTrigger: { trigger: reveal, start: 'top 85%', once: true }
			});
		}
	}

	if (plan.layers.moon) {
		// A slow push-in on the poster. No opacity change, so the poster (the LCP image) is never hidden.
		gsap.to('.layer-moon', { scale: 1.03, duration: 12, ease: 'sine.inOut' });
	}

	if (plan.layers.far) {
		gsap.to('.layer-far', {
			yPercent: -8,
			ease: 'none',
			scrollTrigger: { trigger: root, start: 'top top', end: 'bottom top', scrub: true }
		});
	}

	if (plan.layers.near) {
		gsap.to('.layer-near', {
			yPercent: -16,
			ease: 'none',
			scrollTrigger: { trigger: root, start: 'top top', end: 'bottom top', scrub: true }
		});
	}

	if (plan.layers.smoke) {
		gsap.fromTo(
			'.layer-smoke',
			{ opacity: 0, y: 12 },
			{ opacity: 0.35, y: 0, duration: 2, ease: 'power2.out' }
		);
	}

	if (plan.bats) {
		gsap.fromTo(
			'.layer-bats',
			{ x: -200, opacity: 0 },
			{ x: 200, opacity: 1, duration: 2.4, ease: 'power2.out', repeat: 0 }
		);
	}
}
