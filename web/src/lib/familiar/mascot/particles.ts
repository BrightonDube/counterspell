// Seeded particle plans for the Familiar's crack burst and laugh ring. Pure: no GSAP, no DOM.
// The same seed always gives the same plan, so a replay looks the same and tests can check it.

/** Crack burst particles, from the design (about 40). */
export const CRACK_PARTICLE_COUNT = 40;
/** Embers in the laugh ring. */
export const EMBER_COUNT = 12;
/** Success sparks in the laugh ring. */
export const SPARK_COUNT = 3;
/** Target length of the crack burst, in milliseconds. */
export const CRACK_DURATION_MS = 900;
/** Target length of the laugh ring, in milliseconds. */
export const RING_DURATION_MS = 700;

/** Largest distance a particle starts or travels from the centre, in CSS pixels. */
const MAX_OFFSET = 120;
const MIN_SIZE = 2;
const MAX_SIZE = 6;

/** One particle. Positions are offsets from the centre of the pumpkin, in CSS pixels. */
export interface Particle {
	x: number;
	y: number;
	vx: number;
	vy: number;
	size: number;
	/** Name of the palette token that colours the particle, for example `--danger-fill`. */
	colourToken: string;
}

/** The crack burst: 40 particles and the time they take. */
export interface CrackPlan {
	durationMs: number;
	particles: Particle[];
}

/** The laugh ring: embers, sparks and the time they take. */
export interface RingPlan {
	durationMs: number;
	embers: Particle[];
	sparks: Particle[];
}

/** A small seeded random generator (mulberry32). Returns numbers in [0, 1). */
function seededRandom(seed: number): () => number {
	let state = seed >>> 0;
	return () => {
		state = (state + 0x6d2b79f5) >>> 0;
		let t = state;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

/** A value between `min` and `max` from the generator. */
function between(random: () => number, min: number, max: number): number {
	return min + (max - min) * random();
}

/** One particle with a random start, velocity and size, coloured with `colourToken`. */
function particle(random: () => number, colourToken: string): Particle {
	return {
		x: between(random, -MAX_OFFSET, MAX_OFFSET),
		y: between(random, -MAX_OFFSET, MAX_OFFSET),
		vx: between(random, -3, 3),
		vy: between(random, -3, 3),
		size: between(random, MIN_SIZE, MAX_SIZE),
		colourToken
	};
}

/** The crack-and-burst plan: 40 crimson particles over about 900 ms. */
export function crackPlan(seed: number): CrackPlan {
	const random = seededRandom(seed);
	const particles = Array.from({ length: CRACK_PARTICLE_COUNT }, () =>
		particle(random, '--danger-fill')
	);
	return { durationMs: CRACK_DURATION_MS, particles };
}

/** The laugh-burst plan: 12 embers and 3 green sparks over about 700 ms. */
export function ringPlan(seed: number): RingPlan {
	const random = seededRandom(seed);
	const embers = Array.from({ length: EMBER_COUNT }, () => particle(random, '--accent-candle'));
	const sparks = Array.from({ length: SPARK_COUNT }, () => particle(random, '--success'));
	return { durationMs: RING_DURATION_MS, embers, sparks };
}
