import { cleanup, render, within } from '@testing-library/svelte';
import { afterEach } from 'vitest';
import { describe, expect, it } from 'vitest';
import GhostBeats from './GhostBeats.svelte';
import GhostExample from './GhostExample.svelte';

describe('GhostBeats', () => {
	it('renders three sections with h2 headings Submit, Hunt and Understand in order', () => {
		const { container } = render(GhostBeats);
		const headings = Array.from(container.querySelectorAll('section h2')).map((h) =>
			h.textContent?.trim()
		);
		expect(headings).toEqual(['Submit', 'Hunt', 'Understand']);
	});

	// test-change: R6 (beat icons are now approved art from web/static/art/objects, still hidden from assistive technology)
	it('gives each beat a decorative image from the approved art, hidden from assistive technology', () => {
		const { container } = render(GhostBeats);
		const sections = Array.from(container.querySelectorAll('section'));
		expect(sections.map((s) => s.querySelector('img')?.getAttribute('src'))).toEqual([
			'/art/objects/crypt-key.webp',
			'/art/objects/crystal-ball.webp',
			'/art/objects/brain-jar.webp'
		]);
		for (const section of sections) {
			const art = section.querySelector('img');
			expect(art?.getAttribute('alt')).toBe('');
			expect(art?.getAttribute('aria-hidden')).toBe('true');
			expect(art?.getAttribute('width')).toBe('64');
			expect(art?.getAttribute('height')).toBe('64');
			expect(art?.getAttribute('loading')).toBe('lazy');
		}
	});

	it('places the labelled example inside the Hunt beat', () => {
		const { container } = render(GhostBeats);
		const hunt = Array.from(container.querySelectorAll('section')).find((s) =>
			s.querySelector('h2')?.textContent?.includes('Hunt')
		);
		expect(hunt?.textContent).toContain('Illustration, not a real run.');
	});
});

describe('GhostExample', () => {
	afterEach(() => cleanup());

	it('labels the expected and got values with text, not colour alone', () => {
		const { container } = render(GhostExample);
		const view = within(container);
		expect(view.getByText('expected')).toBeTruthy();
		expect(view.getByText('got')).toBeTruthy();
		expect(view.getByText('[0, 1]')).toBeTruthy();
		expect(view.getByText('[0, 0]')).toBeTruthy();
	});

	it('uses the success token for expected and the danger token for got', () => {
		const { container } = render(GhostExample);
		const view = within(container);
		const expectedValue = view.getByText('[0, 1]');
		const gotValue = view.getByText('[0, 0]');
		expect(expectedValue.className).toContain('text-success');
		expect(gotValue.className).toContain('text-danger');
		expect(container.textContent).toContain('nums = [0, 0], target = 0');
	});

	it('states that the example is an illustration', () => {
		const { container } = render(GhostExample);
		expect(within(container).getByText('Illustration, not a real run.')).toBeTruthy();
	});
});
