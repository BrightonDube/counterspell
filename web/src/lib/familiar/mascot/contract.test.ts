import { describe, expect, it } from 'vitest';
import { REQUIRED_IDS, validate } from './contract';

/** An SVG that carries every required id, and nothing else. */
function svgWith(ids: readonly string[]): string {
	const groups = ids.map((id) => `<g id="${id}"></g>`).join('');
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">${groups}</svg>`;
}

describe('familiar asset contract', () => {
	it('lists the twelve ids the design requires for the mascot', () => {
		expect(REQUIRED_IDS).toEqual([
			'fam-body',
			'fam-glow-stop-inner',
			'fam-glow-stop-outer',
			'fam-eyes-1',
			'fam-eyes-2',
			'fam-eyes-3',
			'fam-eyes-4',
			'fam-eyes-5',
			'fam-mouth-1',
			'fam-mouth-2',
			'fam-mouth-3',
			'fam-mouth-4',
			'fam-mouth-5',
			'fam-cracks',
			'fam-fx'
		]);
	});

	it('accepts an SVG that has every required id', () => {
		expect(validate(svgWith(REQUIRED_IDS))).toEqual({ ok: true, missing: [] });
	});

	it('names the first missing id, so the failure says what to add', () => {
		const withoutCracks = REQUIRED_IDS.filter((id) => id !== 'fam-cracks');
		expect(validate(svgWith(withoutCracks))).toEqual({ ok: false, missing: ['fam-cracks'] });
	});

	it('names every missing id, not only the first', () => {
		const withoutTwo = REQUIRED_IDS.filter((id) => id !== 'fam-eyes-3' && id !== 'fam-mouth-5');
		expect(validate(svgWith(withoutTwo)).missing).toEqual(['fam-eyes-3', 'fam-mouth-5']);
	});

	it('does not accept an id that only appears as a prefix of another id', () => {
		const prefixOnly = REQUIRED_IDS.map((id) => (id === 'fam-fx' ? 'fam-fx-extra' : id));
		expect(validate(svgWith(prefixOnly))).toEqual({ ok: false, missing: ['fam-fx'] });
	});
});
