// @vitest-environment node
// The LCP poster preload lives in app.html so the browser discovers it before the route head.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  HERO_POSTER_SIZES,
  HERO_POSTER_SRCSET
} from './lib/landing/hero-media';

const appHtml = readFileSync(resolve(import.meta.dirname, 'app.html'), 'utf8');

describe('static document head', () => {
  it('preloads the same AVIF candidates and sizes as the hero picture', () => {
    expect(appHtml).toContain(`imagesrcset="${HERO_POSTER_SRCSET}"`);
    expect(appHtml).toContain(`imagesizes="${HERO_POSTER_SIZES}"`);
    expect(appHtml).toContain('fetchpriority="high"');
  });
});
