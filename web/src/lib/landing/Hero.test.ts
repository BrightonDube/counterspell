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

	it('renders five decorative scene layers as images hidden from assistive technology', () => {
		const { container } = render(Hero);
		const images = Array.from(container.querySelectorAll('img'));
		expect(images.map((img) => img.getAttribute('src'))).toEqual([
			'/landing/moon.svg',
			'/landing/graveyard-far.svg',
			'/landing/smoke.svg',
			'/landing/graveyard-near.svg',
			'/landing/bats.svg'
		]);
		for (const img of images) {
			expect(img.getAttribute('alt')).toBe('');
			expect(img.getAttribute('aria-hidden')).toBe('true');
			expect(img.getAttribute('width')).toBeTruthy();
			expect(img.getAttribute('height')).toBeTruthy();
		}
	});
});
