// Tests for the GSAP timelines (efficiency-familiar T30). GSAP is replaced by a fake, so the tests
// check what is created and removed, not how the animation looks.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const gsap = vi.hoisted(() => {
	const timeline = {
		to: vi.fn(),
		fromTo: vi.fn(),
		add: vi.fn(),
		kill: vi.fn()
	};
	timeline.to.mockReturnValue(timeline);
	timeline.fromTo.mockReturnValue(timeline);
	timeline.add.mockReturnValue(timeline);
	return {
		timeline: vi.fn(() => timeline),
		to: vi.fn(),
		fromTo: vi.fn(),
		killTweensOf: vi.fn(),
		tl: timeline
	};
});

vi.mock('gsap', () => ({ default: gsap }));

import { startBurst, startCrossfade } from './timelines';

const SVG = 'http://www.w3.org/2000/svg';

/** A small stand-in for the artwork: the groups the timelines bind to. */
function makeArt(): SVGSVGElement {
	const svg = document.createElementNS(SVG, 'svg') as SVGSVGElement;
	for (const id of ['fam-fx', 'fam-mood-4']) {
		const group = document.createElementNS(SVG, 'g');
		group.setAttribute('id', id);
		svg.appendChild(group);
	}
	document.body.appendChild(svg);
	return svg;
}

function fxGroup(svg: SVGSVGElement): SVGGElement {
	const fx = svg.querySelector('#fam-fx');
	if (fx === null) throw new Error('missing fam-fx');
	return fx as SVGGElement;
}

let art: SVGSVGElement;

beforeEach(() => {
	vi.clearAllMocks();
	gsap.tl.to.mockReturnValue(gsap.tl);
	gsap.tl.fromTo.mockReturnValue(gsap.tl);
	gsap.tl.add.mockReturnValue(gsap.tl);
	art = makeArt();
});

afterEach(() => {
	art.remove();
});

describe('startBurst', () => {
	// Breaks if the crack burst loses particles, so the failure reads as a plain drawing instead of a crack.
	it('adds 40 crimson particles to the fx group for a crack burst', async () => {
		const playback = startBurst(art, 'crack_burst', 7);
		await playback.ready;
		const particles = fxGroup(art).querySelectorAll('circle');
		expect(particles.length).toBe(40);
		expect((particles[0] as SVGElement).style.fill).toBe('var(--danger-fill)');
	});

	// Breaks if the laugh ring has the wrong count, so the success burst looks empty or crowded.
	it('adds 12 embers and 3 green sparks for a laugh burst', async () => {
		const playback = startBurst(art, 'laugh_burst', 7);
		await playback.ready;
		const particles = [...fxGroup(art).querySelectorAll('circle')];
		expect(particles.length).toBe(15);
		const sparks = particles.filter((p) => (p as SVGElement).style.fill === 'var(--success)');
		expect(sparks.length).toBe(3);
	});

	// Breaks if particles stay after the learner leaves or turns motion off, leaving stray shapes in the figure.
	it('removes every particle and kills the timeline on dispose', async () => {
		const playback = startBurst(art, 'crack_burst', 3);
		await playback.ready;
		playback.dispose();
		expect(fxGroup(art).children.length).toBe(0);
		expect(gsap.tl.kill).toHaveBeenCalled();
	});

	// Breaks if a second dispose throws, which would crash the cleanup that runs on every change.
	it('is safe to dispose twice', async () => {
		const playback = startBurst(art, 'laugh_burst', 3);
		await playback.ready;
		playback.dispose();
		expect(() => playback.dispose()).not.toThrow();
	});

	// Breaks if GSAP finishes loading after the learner has left and then draws particles on a dead view.
	it('creates nothing when disposed before GSAP has loaded', async () => {
		const playback = startBurst(art, 'crack_burst', 3);
		playback.dispose();
		await playback.ready;
		expect(fxGroup(art).children.length).toBe(0);
	});
});

describe('startCrossfade', () => {
	// Breaks if the mood change is not faded in, so the expression snaps instead of fading over 300 ms.
	it('fades each shown layer in with an opacity tween', async () => {
		const playback = startCrossfade(art, ['fam-mood-4']);
		await playback.ready;
		expect(gsap.fromTo).toHaveBeenCalledTimes(1);
	});

	// Breaks if a stopped fade leaves a layer half transparent, which would hide the expression.
	it('clears the opacity it set on dispose', async () => {
		const playback = startCrossfade(art, ['fam-mood-4']);
		await playback.ready;
		const layer = art.querySelector('#fam-mood-4') as SVGElement;
		layer.style.opacity = '0.4';
		playback.dispose();
		expect(layer.style.opacity).toBe('');
		expect(gsap.killTweensOf).toHaveBeenCalled();
	});
});
