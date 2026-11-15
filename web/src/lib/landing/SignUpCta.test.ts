import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import SignUpCta from './SignUpCta.svelte';

describe('SignUpCta', () => {
	afterEach(() => cleanup());

	it('links the primary call to action to /login', () => {
		render(SignUpCta);
		const primary = screen.getByRole('link', { name: 'Start practising' });
		expect(primary.getAttribute('href')).toBe('/login');
	});

	it('links the secondary Log in link to /login in the moss accent', () => {
		render(SignUpCta);
		const secondary = screen.getByRole('link', { name: 'Log in' });
		expect(secondary.getAttribute('href')).toBe('/login');
		expect(secondary.className).toContain('text-accent-moss');
	});

	it('styles the primary button with the candle accent and a visible focus ring', () => {
		render(SignUpCta);
		const primary = screen.getByRole('link', { name: 'Start practising' });
		expect(primary.className).toContain('bg-accent-candle');
		expect(primary.className).toContain('hover:bg-accent-candle-glow');
		expect(primary.className).toContain('focus-visible:ring-2');
		expect(primary.className).toContain('focus-visible:ring-offset-2');
		expect(primary.className).toContain('focus-visible:ring-focus');
	});

	it('places a decorative candle beside the call to action, hidden from assistive technology (T17)', () => {
		const { container } = render(SignUpCta);
		const art = container.querySelector('img');
		expect(art?.getAttribute('src')).toBe('/art/objects/candle-flame.webp');
		expect(art?.getAttribute('alt')).toBe('');
		expect(art?.getAttribute('aria-hidden')).toBe('true');
		expect(art?.getAttribute('width')).toBe('96');
		expect(art?.getAttribute('height')).toBe('96');
		expect(art?.getAttribute('loading')).toBe('lazy');
	});

	it('offers no payment or paid tier wording', () => {
		const { container } = render(SignUpCta);
		expect(container.textContent?.toLowerCase()).not.toMatch(/price|pricing|paid|payment|subscri/);
	});
});
