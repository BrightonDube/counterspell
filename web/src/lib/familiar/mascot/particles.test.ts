import { describe, expect, it } from 'vitest';
import {
  CRACK_DURATION_MS,
  CRACK_PARTICLE_COUNT,
  EMBER_COUNT,
  RING_DURATION_MS,
  SPARK_COUNT,
  crackPlan,
  ringPlan
} from './particles';

const MAX_OFFSET = 120;

describe('particle plans', () => {
  it('builds exactly the crack and ring counts', () => {
    expect(crackPlan(7).particles).toHaveLength(CRACK_PARTICLE_COUNT);
    expect(CRACK_PARTICLE_COUNT).toBe(40);
    const ring = ringPlan(7);
    expect(ring.embers).toHaveLength(EMBER_COUNT);
    expect(ring.sparks).toHaveLength(SPARK_COUNT);
    expect(EMBER_COUNT).toBe(12);
    expect(SPARK_COUNT).toBe(3);
  });

  it('keeps every value finite and within its bounds', () => {
    const particles = [
      ...crackPlan(42).particles,
      ...ringPlan(42).embers,
      ...ringPlan(42).sparks
    ];
    for (const p of particles) {
      for (const value of [p.x, p.y, p.vx, p.vy, p.size]) {
        expect(Number.isFinite(value)).toBe(true);
      }
      expect(Math.abs(p.x)).toBeLessThanOrEqual(MAX_OFFSET);
      expect(Math.abs(p.y)).toBeLessThanOrEqual(MAX_OFFSET);
      expect(p.size).toBeGreaterThanOrEqual(2);
      expect(p.size).toBeLessThanOrEqual(6);
      expect(p.colourToken.startsWith('--')).toBe(true);
    }
  });

  it('gives the same plan for the same seed and a different plan for another seed', () => {
    expect(crackPlan(9)).toEqual(crackPlan(9));
    expect(ringPlan(9)).toEqual(ringPlan(9));
    expect(crackPlan(9)).not.toEqual(crackPlan(10));
  });

  it('targets the crack and ring durations within 5%', () => {
    const crack = crackPlan(3).durationMs;
    const ring = ringPlan(3).durationMs;
    expect(
      Math.abs(crack - CRACK_DURATION_MS) / CRACK_DURATION_MS
    ).toBeLessThanOrEqual(0.05);
    expect(
      Math.abs(ring - RING_DURATION_MS) / RING_DURATION_MS
    ).toBeLessThanOrEqual(0.05);
  });
});
