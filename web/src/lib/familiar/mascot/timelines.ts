// GSAP timelines for the mascot: the crack and laugh bursts and the layer crossfade. GSAP is
// imported only when a playback starts, so a static view never loads it. Every playback can be
// disposed, which kills its tweens and removes what it added. Only transform and opacity animate.
import type { Particle } from './particles';
import { crackPlan, ringPlan } from './particles';

type Gsap = (typeof import('gsap'))['default'];
type Timeline = ReturnType<Gsap['timeline']>;

/** A running effect. `ready` settles once GSAP has loaded and the effect has been drawn. */
export interface Playback {
	readonly ready: Promise<void>;
	/** Stops and undoes the effect. Safe to call more than once, and before GSAP has loaded. */
	dispose(): void;
}

/** The two bursts the planner can ask for. */
export type BurstEffect = 'crack_burst' | 'laugh_burst';

const SVG_NS = 'http://www.w3.org/2000/svg';
const CROSSFADE_SECONDS = 0.3;
const CRACK_DRAW_SECONDS = 0.25;
const SHAKE_PIXELS = 3;
const SHAKE_HALF_SECONDS = 0.0625;
const RING_DELAY_SECONDS = 0.1;

/** What one playback owns, so dispose can undo exactly that. */
interface Owned {
	disposed: boolean;
	gsap: Gsap | null;
	timelines: Timeline[];
	/** Elements whose tweens are killed and whose inline styles are reset on dispose. */
	targets: Element[];
	/** Elements this playback created, removed on dispose. */
	nodes: Element[];
}

/** Loads GSAP, then runs `play` unless the playback was already disposed. */
function lazyPlayback(play: (gsap: Gsap, owned: Owned) => void): Playback {
	const owned: Owned = { disposed: false, gsap: null, timelines: [], targets: [], nodes: [] };
	const ready = import('gsap').then(({ default: gsap }) => {
		if (owned.disposed) return;
		owned.gsap = gsap;
		play(gsap, owned);
	});
	return {
		ready,
		dispose() {
			if (owned.disposed) return;
			owned.disposed = true;
			for (const timeline of owned.timelines) timeline.kill();
			owned.gsap?.killTweensOf(owned.targets);
			for (const target of owned.targets) {
				const styled = target as HTMLElement | SVGElement;
				styled.style.transform = '';
				styled.style.opacity = '';
				styled.style.strokeDashoffset = '';
			}
			for (const node of owned.nodes) node.remove();
		}
	};
}

/** Finds a group by id inside the artwork, or null when the artwork lacks it. */
function groupById(art: SVGSVGElement, id: string): SVGGElement | null {
	return art.querySelector<SVGGElement>(`#${id}`);
}

/** Creates one circle per particle at the centre of the artwork, filled with its palette token. */
function spawn(parent: SVGGElement, particles: Particle[], owned: Owned): SVGCircleElement[] {
	return particles.map((particle) => {
		const circle = document.createElementNS(SVG_NS, 'circle');
		circle.setAttribute('cx', '100');
		circle.setAttribute('cy', '100');
		circle.setAttribute('r', String(particle.size));
		circle.style.fill = `var(${particle.colourToken})`;
		parent.appendChild(circle);
		owned.nodes.push(circle);
		return circle;
	});
}

/** Moves each circle to its own offset and fades it out. Values are looked up by index. */
function scatter(
	timeline: Timeline,
	circles: SVGCircleElement[],
	particles: Particle[],
	durationSeconds: number,
	at: number
): void {
	timeline.to(
		circles,
		{
			x: (index: number) => particles[index]?.x ?? 0,
			y: (index: number) => particles[index]?.y ?? 0,
			opacity: 0,
			duration: durationSeconds,
			ease: 'power2.in'
		},
		at
	);
}

/** The crack-and-burst: the crack lines draw on, then the crimson particles fly out. */
function playCrack(gsap: Gsap, art: SVGSVGElement, owned: Owned, seed: number): void {
	const fx = groupById(art, 'fam-fx');
	if (fx === null) return;
	const timeline = gsap.timeline();
	owned.timelines.push(timeline);
	owned.targets.push(art);
	const cracks = groupById(art, 'fam-cracks');
	if (cracks !== null) {
		const paths = [...cracks.querySelectorAll('path')];
		owned.targets.push(...paths);
		timeline.fromTo(paths, { strokeDashoffset: 1 }, { strokeDashoffset: 0, duration: CRACK_DRAW_SECONDS });
	}
	const plan = crackPlan(seed);
	const circles = spawn(fx, plan.particles, owned);
	owned.targets.push(...circles);
	scatter(timeline, circles, plan.particles, plan.durationMs / 1000, CRACK_DRAW_SECONDS);
}

/** The laugh burst: one short shake, then a ring of embers and green sparks. */
function playLaugh(gsap: Gsap, art: SVGSVGElement, owned: Owned, seed: number): void {
	const fx = groupById(art, 'fam-fx');
	if (fx === null) return;
	const timeline = gsap.timeline();
	owned.timelines.push(timeline);
	owned.targets.push(art);
	timeline.to(art, { x: SHAKE_PIXELS, duration: SHAKE_HALF_SECONDS, yoyo: true, repeat: 1 });
	const plan = ringPlan(seed);
	const particles = [...plan.embers, ...plan.sparks];
	const circles = spawn(fx, particles, owned);
	owned.targets.push(...circles);
	scatter(timeline, circles, particles, plan.durationMs / 1000, RING_DELAY_SECONDS);
}

/** Starts one burst on the artwork. The caller disposes it when the view changes. */
export function startBurst(art: SVGSVGElement, effect: BurstEffect, seed: number): Playback {
	return lazyPlayback((gsap, owned) => {
		if (effect === 'crack_burst') {
			playCrack(gsap, art, owned, seed);
		} else {
			playLaugh(gsap, art, owned, seed);
		}
	});
}

/** Fades the named layers in over 300 ms. Layers that are not in the artwork are skipped. */
export function startCrossfade(art: SVGSVGElement, layerIds: string[]): Playback {
	return lazyPlayback((gsap, owned) => {
		for (const id of layerIds) {
			const layer = groupById(art, id);
			if (layer === null) continue;
			owned.targets.push(layer);
			gsap.fromTo(layer, { opacity: 0 }, { opacity: 1, duration: CROSSFADE_SECONDS });
		}
	});
}
