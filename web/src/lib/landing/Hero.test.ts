import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import Hero from './Hero.svelte';
import type { LandingPlan } from './motion-plan';

const STATIC: LandingPlan = {
	mode: 'static',
	loadGsap: false,
	layers: { moon: true, far: true, near: true, smoke: false },
	bats: false,
	reveals: false
};

const MOTION: LandingPlan = {
	mode: 'motion',
	loadGsap: true,
	layers: { moon: true, far: true, near: true, smoke: true },
	bats: true,
	reveals: true
};

describe('Hero', () => {
	it('renders the approved headline as the page h1', () => {
		render(Hero);
		expect(
			screen.getByRole('heading', { level: 1, name: 'Learn why your code breaks.' })
		).toBeTruthy();
	});

	it('shows the supporting line and the eyebrow', () => {
		const { container } = render(Hero);
		expect(container.textContent).toContain('Counterspell');
		expect(container.textContent).toContain('hunts down the input that breaks it');
	});

	// test-change: R5 (the hero now uses the approved moon and graveyard art; design.md amendment 15 Nov 2026)
	it('shows the poster as a WebP picture with both sizes, explicit dimensions and no alt text', () => {
		const { container } = render(Hero);
		const source = container.querySelector('picture source');
		expect(source?.getAttribute('type')).toBe('image/webp');
		expect(source?.getAttribute('srcset')).toBe(
			'/art/landing/moon-graveyard-poster-960.webp 960w, /art/landing/moon-graveyard-poster-1920.webp 1920w'
		);

		const poster = container.querySelector('picture img');
		expect(poster?.getAttribute('src')).toBe('/art/landing/moon-graveyard-poster-1920.jpg');
		expect(poster?.getAttribute('width')).toBe('1920');
		expect(poster?.getAttribute('height')).toBe('1080');
		expect(poster?.getAttribute('alt')).toBe('');
		expect(poster?.getAttribute('aria-hidden')).toBe('true');
		expect(poster?.getAttribute('fetchpriority')).toBe('high');
	});

	it('does not mount the video before first paint', () => {
		const { container } = render(Hero);
		expect(container.querySelector('video')).toBeNull();
	});

	it('shows the fog and embers overlays in the static plan, and no smoke or bats (T16)', () => {
		const { container } = render(Hero, { props: { plan: STATIC } });
		const sources = Array.from(container.querySelectorAll('img')).map((img) =>
			img.getAttribute('src')
		);
		expect(sources).toContain('/art/fx/fog-layer-cold.webp');
		expect(sources).toContain('/art/fx/embers.webp');
		expect(sources).not.toContain('/art/fx/smoke-plume-ash.webp');
		expect(sources).not.toContain('/art/fx/bat-swarm.webp');
	});

	it('adds smoke and bats in the motion plan, hidden until GSAP sets them (T16)', () => {
		const { container } = render(Hero, { props: { plan: MOTION } });
		const smoke = container.querySelector('img.layer-smoke');
		const bats = container.querySelector('img.layer-bats');
		expect(smoke?.getAttribute('src')).toBe('/art/fx/smoke-plume-ash.webp');
		expect(bats?.getAttribute('src')).toBe('/art/fx/bat-swarm.webp');
		// Starts invisible so the overlays do not pop in before the scene timelines load.
		expect(smoke?.getAttribute('style')).toContain('opacity: 0');
		expect(bats?.getAttribute('style')).toContain('opacity: 0');
	});

	it('drops the far fog layer when the plan turns it off (phone rule, R26)', () => {
		const sourcesOf = (container: HTMLElement) =>
			Array.from(container.querySelectorAll('img')).map((img) => img.getAttribute('src'));
		const desktop = render(Hero, { props: { plan: MOTION } });
		expect(sourcesOf(desktop.container)).toContain('/art/fx/fog-layer-cold.webp');
		desktop.unmount();

		const phone: LandingPlan = { ...MOTION, layers: { ...MOTION.layers, far: false }, bats: false };
		const { container } = render(Hero, { props: { plan: phone } });
		expect(sourcesOf(container)).not.toContain('/art/fx/fog-layer-cold.webp');
	});

	it('shows the flat moon and graveyard SVGs when the poster fails to load (T16 fallback)', async () => {
		const { container } = render(Hero, { props: { plan: STATIC } });
		const poster = container.querySelector('picture img') as HTMLImageElement;
		await fireEvent.error(poster);
		const sources = Array.from(container.querySelectorAll('img')).map((img) =>
			img.getAttribute('src')
		);
		expect(sources).toContain('/landing/moon.svg');
		expect(sources).toContain('/landing/graveyard-far.svg');
		expect(sources).toContain('/landing/graveyard-near.svg');
	});
});
