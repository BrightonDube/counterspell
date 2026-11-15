// Component tests for the mascot (efficiency-familiar T29). Each test names the break it catches.
import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Mood } from '../types';
import Mascot from './Mascot.svelte';

const HERE = dirname(fileURLToPath(import.meta.url));
const PLACEHOLDER = readFileSync(
	join(HERE, '..', '..', '..', '..', 'static', 'familiar', 'jack-o-lantern.placeholder.svg'),
	'utf8'
);
const SOURCE = readFileSync(join(HERE, 'Mascot.svelte'), 'utf8');

/** Finds a drawn layer by id, failing the test with the id if the art lacks it. */
function layer(id: string): HTMLElement | SVGElement {
	const found = document.getElementById(id);
	if (found === null) throw new Error(`missing layer ${id}`);
	return found;
}

function props(mood: Mood, overrides: Record<string, unknown> = {}) {
	return {
		mood,
		reason: 'measured',
		headline: 'Looks linear, medium confidence. Reference linear.',
		badge: 'Matches reference!',
		...overrides
	};
}

/** Waits until the artwork has been drawn into the mascot. */
async function artDrawn(): Promise<void> {
	await waitFor(() => expect(document.getElementById('fam-body')).not.toBeNull());
}

beforeEach(() => {
	vi.stubGlobal(
		'fetch',
		vi.fn(async () => ({ ok: true, text: async () => PLACEHOLDER }))
	);
});

afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});

describe('Mascot static rendering', () => {
	// Breaks if a mood shows the wrong expression, so the learner sees a face that does not match the measured result.
	it('shows only the eyes and mouth for the current mood', async () => {
		for (const mood of [1, 2, 3, 4, 5] as Mood[]) {
			const view = render(Mascot, { props: props(mood) });
			await artDrawn();
			for (const other of [1, 2, 3, 4, 5]) {
				const eyes = layer(`fam-eyes-${other}`);
				const mouth = layer(`fam-mouth-${other}`);
				const shown = other === mood;
				expect(eyes.style.display === 'none').toBe(!shown);
				expect(mouth.style.display === 'none').toBe(!shown);
			}
			view.unmount();
		}
	});

	// Breaks if the headline or badge is dropped, so the result is only visible as a face.
	it('shows the headline and badge as text', async () => {
		render(Mascot, { props: props(4) });
		await artDrawn();
		expect(screen.getByText('Looks linear, medium confidence. Reference linear.')).toBeTruthy();
		expect(screen.getByText('Matches reference!')).toBeTruthy();
	});

	// Breaks if the headline is injected as HTML, so a program's output could run markup in the browser.
	it('renders an HTML-looking headline as plain text', async () => {
		const markup = '<img src=x onerror="alert(1)">';
		const view = render(Mascot, { props: props(3, { headline: markup }) });
		await artDrawn();
		expect(screen.getByText(markup)).toBeTruthy();
		expect(view.container.querySelector('img')).toBeNull();
	});

	// Breaks if the live text is not in the required form, so screen readers announce something other than the measured result.
	it('announces the result in a polite live region in the required form', async () => {
		render(Mascot, { props: props(4) });
		await artDrawn();
		const live = document.querySelector('[aria-live="polite"]');
		expect(live?.textContent).toBe(
			'Familiar: pleased. Looks linear, medium confidence. Reference linear.'
		);
	});

	// Breaks if hiding removes the screen-reader text too, so hidden learners miss the result.
	it('hides the face but keeps a keyboard button and the live text when hidden', async () => {
		render(Mascot, { props: props(2, { hidden: true, onToggleHidden: () => {} }) });
		expect(document.getElementById('fam-body')).toBeNull();
		const button = screen.getByRole('button', { name: 'Show Familiar' });
		expect(button.tagName).toBe('BUTTON');
		expect(button.className).toContain('min-h-11');
		expect(button.className).toContain('min-w-11');
		expect(document.querySelector('[aria-live="polite"]')?.textContent).toContain('Familiar: angry.');
	});

	// Breaks if missing artwork throws or hides the badge, so learners still see the result as text.
	it('shows the badge text when the artwork cannot be loaded', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false, text: async () => '' })));
		render(Mascot, { props: props(1) });
		await waitFor(() => expect(fetch).toHaveBeenCalled());
		expect(screen.getByText('Matches reference!')).toBeTruthy();
		expect(document.getElementById('fam-body')).toBeNull();
	});

	// Breaks if artwork missing a required id is drawn, because the layers would then silently do nothing.
	it('does not draw artwork that breaks the id contract', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () => ({ ok: true, text: async () => '<svg id="fam-body"></svg>' }))
		);
		render(Mascot, { props: props(5) });
		await waitFor(() => expect(fetch).toHaveBeenCalled());
		await new Promise((resolve) => setTimeout(resolve, 0));
		expect(document.getElementById('fam-eyes-5')).toBeNull();
	});

	// Breaks if the component writes HTML from strings or handles image errors inline, both common routes for injection.
	it('uses no innerHTML, {@html} or onerror in the component source', () => {
		expect(SOURCE).not.toMatch(/\{@html/);
		expect(SOURCE).not.toMatch(/innerHTML/);
		expect(SOURCE).not.toMatch(/onerror/i);
	});

	// Breaks if GSAP loads with the static view, which the design forbids: GSAP loads only for planned effects.
	it('does not import GSAP in the component', () => {
		expect(SOURCE).not.toMatch(/gsap/);
	});
});
