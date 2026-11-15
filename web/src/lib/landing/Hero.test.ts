import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import Hero from './Hero.svelte';

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
});
