import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import Page from './+page.svelte';

describe('landing route composition (T08)', () => {
	afterEach(() => cleanup());

	it('keeps the smoke marker on the root main element with lang en', () => {
		const { container } = render(Page);
		const main = container.querySelector('main');
		expect(main?.getAttribute('data-smoke-marker')).toBe('counterspell-svelte-static');
		expect(main?.getAttribute('lang')).toBe('en');
	});

	it('shows the headline, the three beat headings in order and the sign-up link', () => {
		const { container } = render(Page);
		expect(screen.getByRole('heading', { level: 1 }).textContent?.trim()).toBe(
			'Learn why your code breaks.'
		);
		const beats = Array.from(container.querySelectorAll('section h2')).map((h) =>
			h.textContent?.trim()
		);
		expect(beats).toEqual(['Submit', 'Hunt', 'Understand']);
		expect(screen.getByRole('link', { name: 'Start practising' }).getAttribute('href')).toBe(
			'/login'
		);
	});

	it('no longer shows the skeleton milestone message', () => {
		const { container } = render(Page);
		expect(container.textContent).not.toContain('Skeleton milestone');
	});
});
