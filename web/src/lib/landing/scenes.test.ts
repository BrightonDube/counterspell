import { describe, expect, it, vi } from 'vitest';
import type { LandingPlan } from './motion-plan';
import { startScenes, type SceneDeps } from './scenes';

const MOTION: LandingPlan = {
	mode: 'motion',
	loadGsap: true,
	layers: { moon: true, far: true, near: true, smoke: true },
	bats: true,
	reveals: true
};

const STATIC: LandingPlan = {
	mode: 'static',
	loadGsap: false,
	layers: { moon: true, far: true, near: true, smoke: false },
	bats: false,
	reveals: false
};

interface Call {
	method: 'from' | 'fromTo' | 'to' | 'registerPlugin' | 'context';
	target?: string;
	vars?: Record<string, unknown>;
	from?: Record<string, unknown>;
}

/** A recording stand-in for GSAP. It records calls and never animates anything. */
function fakeGsap() {
	const calls: Call[] = [];
	const revert = vi.fn();
	const targetName = (t: unknown) => (typeof t === 'string' ? t : (t as { selector?: string }).selector ?? 'element');
	const gsap = {
		registerPlugin: (...plugins: unknown[]) => calls.push({ method: 'registerPlugin', vars: { count: plugins.length } }),
		context: (fn: () => void) => {
			calls.push({ method: 'context' });
			fn();
			return { revert };
		},
		from: (target: unknown, vars: Record<string, unknown>) =>
			calls.push({ method: 'from', target: targetName(target), vars }),
		fromTo: (target: unknown, from: Record<string, unknown>, vars: Record<string, unknown>) =>
			calls.push({ method: 'fromTo', target: targetName(target), vars, from }),
		to: (target: unknown, vars: Record<string, unknown>) =>
			calls.push({ method: 'to', target: targetName(target), vars })
	};
	return { gsap, calls, revert };
}

/** A root with the scene layers and reveal targets the page renders. */
function sceneRoot(): HTMLElement {
	const root = document.createElement('main');
	for (const name of ['moon', 'far', 'smoke', 'near', 'bats']) {
		const img = document.createElement('img');
		img.className = `layer layer-${name}`;
		root.appendChild(img);
	}
	const reveal = document.createElement('section');
	reveal.setAttribute('data-reveal', '');
	root.appendChild(reveal);
	document.body.appendChild(root);
	return root;
}

function deps(gsapStub: ReturnType<typeof fakeGsap>, loaded = { count: 0 }): SceneDeps {
	return {
		schedule: (cb) => cb(),
		load: async () => {
			loaded.count += 1;
			return { gsap: gsapStub.gsap as never, ScrollTrigger: {} as never };
		}
	};
}

describe('startScenes', () => {
	it('never loads GSAP for a static plan (R14, reduced motion)', async () => {
		const stub = fakeGsap();
		const loaded = { count: 0 };
		const dispose = await startScenes(STATIC, document.createElement('main'), deps(stub, loaded));
		expect(loaded.count).toBe(0);
		expect(stub.calls).toHaveLength(0);
		expect(() => dispose()).not.toThrow();
	});

	it('waits for the schedule callback before it loads GSAP after first paint (R14)', async () => {
		const stub = fakeGsap();
		const loaded = { count: 0 };
		let scheduled: (() => void) | undefined;
		const pending = startScenes(MOTION, sceneRoot(), {
			schedule: (cb) => (scheduled = cb),
			load: async () => {
				loaded.count += 1;
				return { gsap: stub.gsap as never, ScrollTrigger: {} as never };
			}
		});
		expect(loaded.count).toBe(0);
		scheduled?.();
		await pending;
		expect(loaded.count).toBe(1);
	});

	it('uses a scroll reveal that does not hide content before the trigger fires (R14, immediateRender false)', async () => {
		const stub = fakeGsap();
		await startScenes(MOTION, sceneRoot(), deps(stub));
		const reveals = stub.calls.filter((c) => c.method === 'from' && c.vars?.scrollTrigger);
		expect(reveals.length).toBeGreaterThan(0);
		for (const reveal of reveals) {
			expect(reveal.vars?.immediateRender).toBe(false);
			expect((reveal.vars?.scrollTrigger as { once: boolean }).once).toBe(true);
		}
	});

	it('animates only transform and opacity (no layout properties)', async () => {
		const stub = fakeGsap();
		await startScenes(MOTION, sceneRoot(), deps(stub));
		const allowed = new Set([
			'opacity',
			'y',
			'yPercent',
			'x',
			'xPercent',
			'scale',
			'duration',
			'ease',
			'delay',
			'repeat',
			'immediateRender',
			'scrollTrigger',
			'stagger',
			'count'
		]);
		for (const call of stub.calls) {
			for (const key of Object.keys({ ...call.vars, ...call.from })) {
				expect(allowed.has(key), `unexpected property ${key}`).toBe(true);
			}
		}
	});

	it('skips the far layer and bats when the plan turns them off (phone rule)', async () => {
		const stub = fakeGsap();
		const phone: LandingPlan = { ...MOTION, layers: { ...MOTION.layers, far: false }, bats: false };
		await startScenes(phone, sceneRoot(), deps(stub));
		const targets = stub.calls.map((c) => c.target);
		expect(targets).not.toContain('.layer-far');
		expect(targets).not.toContain('.layer-bats');
	});

	it('never starts an infinite loop (R13)', async () => {
		const stub = fakeGsap();
		await startScenes(MOTION, sceneRoot(), deps(stub));
		for (const call of stub.calls) {
			expect(call.vars?.repeat ?? 0).toBe(0);
		}
	});

	it('returns a dispose function that reverts the GSAP context (route leave)', async () => {
		const stub = fakeGsap();
		const dispose = await startScenes(MOTION, sceneRoot(), deps(stub));
		expect(stub.revert).not.toHaveBeenCalled();
		dispose();
		expect(stub.revert).toHaveBeenCalledTimes(1);
	});

	it('moves the poster with scale only, so the LCP image is never hidden (T16)', async () => {
		const stub = fakeGsap();
		await startScenes(MOTION, sceneRoot(), deps(stub));
		const moon = stub.calls.find((c) => c.target === '.layer-moon');
		expect(moon?.vars?.scale).toBe(1.03);
		expect(moon?.vars?.opacity).toBeUndefined();
		expect(moon?.from?.opacity).toBeUndefined();
	});
});
