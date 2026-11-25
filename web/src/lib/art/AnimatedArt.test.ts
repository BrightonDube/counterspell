// AnimatedArt motion contract (WP05/WP07): the animated loop only mounts while
// the art is visible and full motion is allowed. Reduced-motion visits get the
// still poster frame and never create a video element.
import { cleanup, render } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import AnimatedArt from './AnimatedArt.svelte';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/** Stubs matchMedia so the prefers-reduced-motion query answers `reduced`. */
function stubReducedMotion(reduced: boolean) {
  vi.stubGlobal(
    'matchMedia',
    vi.fn((query: string) => ({
      matches: query.includes('prefers-reduced-motion') ? reduced : false,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn()
    }))
  );
}

describe('AnimatedArt reduced-motion contract', () => {
  // Breaks if reduced-motion visits fetch or play the loop anyway.
  it('mounts only the still image when reduced motion is preferred', async () => {
    stubReducedMotion(true);
    const { container } = render(AnimatedArt, {
      props: { src: '/art/a.webp', webm: '/art/a.webm' }
    });
    await vi.waitFor(() => {
      expect(container.querySelector('img')).not.toBeNull();
    });
    expect(container.querySelector('video')).toBeNull();
  });

  // Breaks if full-motion visits lose the loop — the costume must be preserved,
  // not silently degraded. (happy-dom's IntersectionObserver never reports, so
  // stub it out to reach the treated-as-visible fallback.)
  it('mounts the loop when motion is allowed', async () => {
    stubReducedMotion(false);
    vi.stubGlobal('IntersectionObserver', undefined);
    const { container } = render(AnimatedArt, {
      props: { src: '/art/a.webp', webm: '/art/a.webm' }
    });
    await vi.waitFor(() => {
      expect(container.querySelector('video')).not.toBeNull();
    });
    expect(container.querySelector('video source')?.getAttribute('src')).toBe(
      '/art/a.webm'
    );
  });

  // Breaks if the still fallback loses its lazy/async loading hints.
  it('keeps lazy loading hints on the still fallback', () => {
    stubReducedMotion(true);
    const { container } = render(AnimatedArt, {
      props: { src: '/art/a.webp', webm: '/art/a.webm' }
    });
    const img = container.querySelector('img');
    expect(img?.getAttribute('loading')).toBe('lazy');
    expect(img?.getAttribute('decoding')).toBe('async');
  });
});
