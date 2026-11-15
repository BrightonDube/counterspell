// Component tests for the mascot (efficiency-familiar T29). Each test names the break it catches.
import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Mood } from '../types';
import Mascot from './Mascot.svelte';

// GSAP is replaced so the tests can see whether any animation was started.
const gsap = vi.hoisted(() => {
	const timeline = { to: vi.fn(), fromTo: vi.fn(), kill: vi.fn() };
	timeline.to.mockReturnValue(timeline);
	timeline.fromTo.mockReturnValue(timeline);
	return { timeline: vi.fn(() => timeline), fromTo: vi.fn(), killTweensOf: vi.fn() };
});
vi.mock('gsap', () => ({ default: gsap }));

const HERE = dirname(fileURLToPath(import.meta.url));
const ART = readFileSync(
	join(HERE, '..', '..', '..', '..', 'static', 'familiar', 'jack-o-lantern.svg'),
	'utf8'
);
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
	vi.clearAllMocks();
	vi.stubGlobal(
		'fetch',
		vi.fn(async () => ({ ok: true, text: async () => ART }))
	);
});

afterEach(() => {
	cleanup();
	vi.unstubAllGlobals();
});

describe('Mascot static rendering', () => {
	// Breaks if a mood shows the wrong painting, so the learner sees a face that does not match the measured result.
	it('shows only the painted image for the current mood', async () => {
		for (const mood of [1, 2, 3, 4, 5] as Mood[]) {
			const view = render(Mascot, { props: props(mood) });
			await artDrawn();
			for (const other of [1, 2, 3, 4, 5]) {
				const painting = layer(`fam-mood-${other}`);
				expect(painting.style.opacity).toBe(other === mood ? '1' : '0');
			}
			view.unmount();
		}
	});

	// Breaks if the realistic art is ignored and the placeholder is drawn while the real file is present.
	it('draws the approved art when it loads, and the placeholder only when it does not', async () => {
		render(Mascot, { props: props(2) });
		await artDrawn();
		expect(fetch).toHaveBeenCalledWith('/familiar/jack-o-lantern.svg');
		expect(fetch).not.toHaveBeenCalledWith('/familiar/jack-o-lantern.placeholder.svg');
	});

	// Breaks if a missing real file leaves no art at all, so the fallback never appears.
	it('falls back to the placeholder when the approved art cannot be loaded', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) =>
				url.endsWith('placeholder.svg')
					? { ok: true, text: async () => PLACEHOLDER }
					: { ok: false, text: async () => '' }
			)
		);
		render(Mascot, { props: props(3) });
		await artDrawn();
		expect(fetch).toHaveBeenCalledWith('/familiar/jack-o-lantern.placeholder.svg');
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
		expect(document.getElementById('fam-mood-5')).toBeNull();
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

/** Lets any pending dynamic import settle before the test checks what was created. */
async function settle(): Promise<void> {
	await new Promise((resolve) => setTimeout(resolve, 20));
}

function fxChildren(): number {
	return document.getElementById('fam-fx')?.children.length ?? 0;
}

describe('Mascot effects', () => {
	// Breaks if a live burst never plays, so a failed submission shows no crack-and-burst.
	it('plays the crack burst for a live result', async () => {
		render(Mascot, { props: props(1, { effect: 'crack_burst', live: true }) });
		await waitFor(() => expect(fxChildren()).toBe(40));
	});

	// Breaks if reduced motion still starts GSAP, which the learner turned off for comfort.
	it('never starts GSAP under reduced motion, and shows the badge text instead', async () => {
		render(Mascot, { props: props(1, { effect: 'crack_burst', live: true, reducedMotion: true }) });
		await artDrawn();
		await settle();
		expect(gsap.timeline).not.toHaveBeenCalled();
		expect(gsap.fromTo).not.toHaveBeenCalled();
		expect(fxChildren()).toBe(0);
		expect(screen.getByText('Matches reference!')).toBeTruthy();
	});

	// Breaks if a live mood change snaps instead of cross-fading, or fades the wrong painting.
	it('cross-fades only the new mood image on a live change', async () => {
		const view = render(Mascot, { props: props(2, { live: true }) });
		await artDrawn();
		await view.rerender(props(3, { live: true }));
		await waitFor(() => expect(gsap.fromTo).toHaveBeenCalled());
		const targets = gsap.fromTo.mock.calls.map((call) => (call[0] as Element).id);
		expect(targets.every((id) => id === 'fam-mood-3')).toBe(true);
		// The fade tween is mocked, so the new image stays at its starting opacity of 0 here.
		expect(layer('fam-mood-3').style.opacity).toBe('0');
	});

	// Breaks if muting still animates, so a muted learner gets motion they asked to stop.
	it('never starts GSAP when muted', async () => {
		render(Mascot, { props: props(5, { effect: 'laugh_burst', live: true, muted: true }) });
		await artDrawn();
		await settle();
		expect(gsap.timeline).not.toHaveBeenCalled();
		expect(gsap.fromTo).not.toHaveBeenCalled();
	});

	// Breaks if a loaded result replays its burst, so every page reload would look like a new result.
	it('does not play a burst for a result loaded from history', async () => {
		render(Mascot, { props: props(1, { effect: 'crack_burst', live: false }) });
		await artDrawn();
		await settle();
		expect(fxChildren()).toBe(0);
	});

	// Breaks if turning reduced motion on does not stop a burst already running, so motion continues after the setting changes.
	it('removes a running burst when reduced motion is switched on', async () => {
		const view = render(Mascot, { props: props(1, { effect: 'crack_burst', live: true }) });
		await waitFor(() => expect(fxChildren()).toBe(40));
		await view.rerender(props(1, { effect: 'crack_burst', live: true, reducedMotion: true }));
		expect(fxChildren()).toBe(0);
	});

	// Breaks if two bursts run within 2 s, which would flash the figure on every quick result.
	it('skips a second burst that starts within 2 seconds of the first', async () => {
		const clock = vi.spyOn(performance, 'now').mockReturnValue(0);
		const view = render(Mascot, { props: props(1, { effect: 'crack_burst', live: true }) });
		await waitFor(() => expect(fxChildren()).toBe(40));
		clock.mockReturnValue(1000);
		await view.rerender(props(5, { effect: 'laugh_burst', live: true }));
		await settle();
		expect(fxChildren()).toBe(0);
		expect(screen.getByText('Matches reference!')).toBeTruthy();
		clock.mockRestore();
	});
});

describe('Mascot mute, hide and reduced motion (T31)', () => {
	// Breaks if muting hides the words, so a muted learner loses the measured result.
	it('keeps the headline visible when muted, with a static face', async () => {
		render(Mascot, { props: props(4, { muted: true }) });
		await artDrawn();
		expect(screen.getByText('Looks linear, medium confidence. Reference linear.')).toBeTruthy();
		expect(document.getElementById('fam-mood-4')).not.toBeNull();
	});

	// Breaks if the show button does nothing, so a hidden learner cannot bring the figure back.
	it('calls the parent when the hidden dot is pressed', async () => {
		const onToggleHidden = vi.fn();
		render(Mascot, { props: props(2, { hidden: true, onToggleHidden }) });
		await screen.getByRole('button', { name: 'Show Familiar' }).click();
		expect(onToggleHidden).toHaveBeenCalledTimes(1);
	});

	// Breaks if reduced motion removes the failure text, so the learner gets no signal once the burst is skipped.
	it('shows the badge text and no particles for a failure under reduced motion', async () => {
		render(Mascot, {
			props: props(1, {
				effect: 'crack_burst',
				live: true,
				reducedMotion: true,
				badge: 'Timeout',
				failed: true
			})
		});
		await artDrawn();
		expect(screen.getByText('Timeout')).toBeTruthy();
		expect(fxChildren()).toBe(0);
	});
});
