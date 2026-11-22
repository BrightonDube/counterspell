// @vitest-environment node
// The LCP poster preload lives only on the landing route so other pages do not download it.
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  HERO_POSTER_SIZES,
  HERO_POSTER_SRCSET
} from './lib/landing/hero-media';

const appHtml = readFileSync(resolve(import.meta.dirname, 'app.html'), 'utf8');
const landingRoute = readFileSync(
  resolve(import.meta.dirname, 'routes/+page.svelte'),
  'utf8'
);

describe('landing document head', () => {
  it('preloads the same AVIF candidates and sizes as the hero picture', () => {
    expect(landingRoute).toContain('imagesrcset={HERO_POSTER_SRCSET}');
    expect(landingRoute).toContain('imagesizes={HERO_POSTER_SIZES}');
    expect(landingRoute).toContain('fetchpriority="high"');
    expect(HERO_POSTER_SRCSET).toContain('1464w');
    expect(HERO_POSTER_SIZES).toBe('calc(100vh * 1.7778)');
  });

  it('keeps the global document head free of landing art', () => {
    expect(appHtml).not.toContain('hero-moon');
  });
});
