// Tests for the authored hint adapter and the hint panel (ide-workspace T17). Each test names the break it catches.
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import { requestAuthoredHint } from './hints';
import { ApiRequestError } from './client';
import HintPanel from '../workspace/HintPanel.svelte';
import { initialLadder, receiveRung, requestNext } from '../workspace/hint-ladder';

afterEach(() => {
	cleanup();
});

function jsonResponse(body: unknown): Response {
	return new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } });
}

describe('requestAuthoredHint', () => {
	// Breaks if the rung or submission is sent in the wrong shape, so the server reveals the wrong hint.
	it('posts the submission and the requested rung', async () => {
		let body: unknown;
		const fetchImpl = async (_url: string, init?: RequestInit) => {
			body = JSON.parse(init?.body as string);
			return jsonResponse({ rung: 2, text: 'Look at the inner loop.' });
		};
		const hint = await requestAuthoredHint('sub-1', 2, { fetchImpl });
		expect(body).toEqual({ submission_id: 'sub-1', rung: 2 });
		expect(hint).toEqual({ rung: 2, text: 'Look at the inner loop.' });
	});

	// Breaks if a hint for a different rung is accepted, which would put the wrong text under the wrong number.
	it('refuses a reply for a different rung', async () => {
		const fetchImpl = async () => jsonResponse({ rung: 3, text: 'Too far ahead.' });
		await expect(requestAuthoredHint('sub-1', 2, { fetchImpl })).rejects.toBeInstanceOf(ApiRequestError);
	});
});

describe('HintPanel', () => {
	// Breaks if hint text is parsed as markup, which would run whatever the hint contains.
	it('shows hint text as plain text', () => {
		const ladder = receiveRung(requestNext(initialLadder(2)).state, 1, '<img src=x onerror="alert(1)"> nudge');
		const { container } = render(HintPanel, { props: { ladder, onNext: () => {} } });
		expect(container.querySelector('img')).toBeNull();
		expect(screen.getByText(/<img src=x onerror="alert\(1\)"> nudge/)).toBeTruthy();
	});

	// Breaks if a second click is allowed while a hint is loading, which would reveal two rungs from one press.
	it('disables the next-hint button while a rung is loading', async () => {
		let clicks = 0;
		const ladder = requestNext(initialLadder(2)).state;
		render(HintPanel, { props: { ladder, onNext: () => (clicks += 1) } });
		const button = screen.getByRole('button', { name: /loading hint/i }) as HTMLButtonElement;
		expect(button.disabled).toBe(true);
		await fireEvent.click(button);
		expect(clicks).toBe(0);
	});
});
