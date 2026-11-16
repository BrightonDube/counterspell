import { describe, expect, it } from 'vitest';
import { planLanding, type MotionFacts } from './motion-plan';

const WIDTHS = [320, 639, 640, 1280];
const BOOLEANS = [true, false];

/** Every combination of the boolean facts at each width the page must handle. */
function allFacts(): MotionFacts[] {
	const cases: MotionFacts[] = [];
	for (const viewportWidth of WIDTHS) {
		for (const reducedMotion of BOOLEANS) {
			for (const scriptsRunning of BOOLEANS) {
				cases.push({ reducedMotion, scriptsRunning, viewportWidth });
			}
		}
	}
	return cases;
}

describe('planLanding', () => {
	it('returns static mode with no GSAP whenever reduced motion is requested (R15)', () => {
		for (const facts of allFacts().filter((f) => f.reducedMotion)) {
			const plan = planLanding(facts);
			expect(plan.mode, JSON.stringify(facts)).toBe('static');
			expect(plan.loadGsap, JSON.stringify(facts)).toBe(false);
		}
	});

	it('returns static mode with no GSAP when scripts are not running', () => {
		for (const facts of allFacts().filter((f) => !f.scriptsRunning)) {
			const plan = planLanding(facts);
			expect(plan.mode, JSON.stringify(facts)).toBe('static');
			expect(plan.loadGsap, JSON.stringify(facts)).toBe(false);
		}
	});

	it('shows every static layer in its final position, with no bats and no reveals', () => {
		for (const facts of allFacts().filter((f) => f.reducedMotion || !f.scriptsRunning)) {
			const plan = planLanding(facts);
			expect(plan.layers.moon, JSON.stringify(facts)).toBe(true);
			expect(plan.layers.far, JSON.stringify(facts)).toBe(true);
			expect(plan.layers.near, JSON.stringify(facts)).toBe(true);
			expect(plan.bats, JSON.stringify(facts)).toBe(false);
			expect(plan.reveals, JSON.stringify(facts)).toBe(false);
		}
	});

	it('hides the far layer and the bats under 640 px when motion is allowed', () => {
		for (const facts of allFacts().filter(
			(f) => !f.reducedMotion && f.scriptsRunning && f.viewportWidth < 640
		)) {
			const plan = planLanding(facts);
			expect(plan.mode, JSON.stringify(facts)).toBe('motion');
			expect(plan.layers.far, JSON.stringify(facts)).toBe(false);
			expect(plan.bats, JSON.stringify(facts)).toBe(false);
		}
	});

	it('keeps the far layer and bats at 640 px and wider when motion is allowed', () => {
		for (const facts of allFacts().filter(
			(f) => !f.reducedMotion && f.scriptsRunning && f.viewportWidth >= 640
		)) {
			const plan = planLanding(facts);
			expect(plan.layers.far, JSON.stringify(facts)).toBe(true);
			expect(plan.bats, JSON.stringify(facts)).toBe(true);
		}
	});

	it('enables GSAP, reveals and every layer in full motion on a wide screen', () => {
		const plan = planLanding({ reducedMotion: false, scriptsRunning: true, viewportWidth: 1280 });
		expect(plan).toEqual({
			mode: 'motion',
			loadGsap: true,
			layers: { moon: true, far: true, near: true, smoke: true },
			bats: true,
			reveals: true
		});
	});

	it('never puts the headline or the beats into the layer set', () => {
		for (const facts of allFacts()) {
			expect(Object.keys(planLanding(facts).layers).sort()).toEqual(
				['far', 'moon', 'near', 'smoke']
			);
		}
	});
});
