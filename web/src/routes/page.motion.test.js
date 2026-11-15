import { cleanup, render } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const disposeScenes = vi.fn();
/** @type {import('vitest').Mock<(plan: unknown) => Promise<() => void>>} */
const startScenes = vi.fn(async (_plan) => disposeScenes);

vi.mock('../lib/landing/scenes', () => ({
	/** @param {unknown} plan */
	startScenes: (plan) => startScenes(plan)
}));

import Page from './+page.svelte';

/**
 * Stub the reduced-motion media query.
 * @param {boolean} reduced
 */
function setReducedMotion(reduced) {
	window.matchMedia = /** @type {any} */ (
		vi.fn(() => ({
			matches: reduced,
			media: '(prefers-reduced-motion: reduce)',
			addEventListener: () => {},
			removeEventListener: () => {}
		}))
	);
}

describe('landing motion wiring (T10)', () => {
	beforeEach(() => {
		startScenes.mockClear();
		disposeScenes.mockClear();
	});

	afterEach(() => cleanup());

	it('starts the scenes when motion is allowed', async () => {
		setReducedMotion(false);
		render(Page);
		await vi.waitFor(() => expect(startScenes).toHaveBeenCalledTimes(1));
		expect(/** @type {any} */ (startScenes.mock.calls[0])[0]).toMatchObject({ mode: 'motion', loadGsap: true });
	});

	it('does not start the scenes when reduced motion is requested', async () => {
		setReducedMotion(true);
		render(Page);
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(startScenes).not.toHaveBeenCalled();
	});

	it('keeps the static page when the scene module fails to load', async () => {
		setReducedMotion(false);
		startScenes.mockImplementationOnce(async () => {
			throw new Error('chunk failed');
		});
		const { container } = render(Page);
		await vi.waitFor(() => expect(startScenes).toHaveBeenCalledTimes(1));
		await new Promise((resolve) => setTimeout(resolve, 0));
		// test-change: R20 (the wordmark is the page h1; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T23)
		expect(container.querySelector('h1')?.textContent).toContain('Counterspell');
	});

	it('disposes the scenes when the page is left', async () => {
		setReducedMotion(false);
		const { unmount } = render(Page);
		await vi.waitFor(() => expect(startScenes).toHaveBeenCalledTimes(1));
		expect(disposeScenes).not.toHaveBeenCalled();
		unmount();
		expect(disposeScenes).toHaveBeenCalledTimes(1);
	});
});
