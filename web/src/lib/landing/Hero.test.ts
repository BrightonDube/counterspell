import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor
} from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import Hero from './Hero.svelte';
import type { LandingPlan } from './motion-plan';

const STATIC: LandingPlan = {
  mode: 'static',
  loadGsap: false,
  layers: { moon: true, far: true, near: true, smoke: false },
  bats: false,
  reveals: false
};

const MOTION: LandingPlan = {
  mode: 'motion',
  loadGsap: true,
  layers: { moon: true, far: true, near: true, smoke: true },
  bats: true,
  reveals: true
};

const sourcesOf = (container: HTMLElement) =>
  Array.from(container.querySelectorAll('img')).map((img) =>
    img.getAttribute('src')
  );

/**
 * Renders the hero and waits for the overlays. They are added after first paint (T18), so a
 * plain render does not contain them yet. The embers layer is in every plan, so it marks "settled".
 */
async function renderSettled(plan?: LandingPlan) {
  const result = render(Hero, plan ? { props: { plan } } : undefined);
  await waitFor(
    () =>
      expect(result.container.querySelector('img.layer-near')).not.toBeNull(),
    {
      timeout: 3000
    }
  );
  return result;
}

describe('Hero', () => {
  // Each test starts from an empty document, so one test's hero cannot leak into the next.
  afterEach(() => cleanup());
  // test-change: R20 (the title-screen wordmark is the h1; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T23)
  it('renders the Counterspell wordmark as the page h1, as real text and not an image', () => {
    const { container } = render(Hero);
    const heading = screen.getByRole('heading', {
      level: 1,
      name: 'Counterspell'
    });
    expect(heading).toBeTruthy();
    expect(heading.querySelector('img')).toBeNull();
    expect(container.querySelectorAll('h1')).toHaveLength(1);
  });

  // test-change: R21 (the tagline replaces the v2 supporting line under the wordmark; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T23)
  it('shows the tagline as text under the wordmark', () => {
    const { container } = render(Hero);
    expect(container.textContent).toContain(
      'Outthink the Machine. Sharpen Your Mind.'
    );
  });

  // landing-steel-night T24: the ENTER control and the Enter-key shortcut.
  describe('ENTER control (T24)', () => {
    it('is a link to /login with an accessible name', () => {
      render(Hero);
      const enter = screen.getByRole('link', { name: /enter/i });
      expect(enter.getAttribute('href')).toBe('/login');
    });

    it('shows the ENTER label and the press hint as text', () => {
      const { container } = render(Hero);
      expect(
        container.querySelector('a[href="/login"]')?.textContent
      ).toContain('ENTER');
      expect(
        container.querySelector('a[href="/login"]')?.textContent
      ).toContain('Press to begin');
    });

    it('follows the ENTER link when Enter is pressed and nothing is focused', () => {
      render(Hero);
      const enter = screen.getByRole('link', { name: /enter/i });
      const onClick = (event: Event) => event.preventDefault();
      enter.addEventListener('click', onClick);
      let clicks = 0;
      enter.addEventListener('click', () => clicks++);
      (document.activeElement as HTMLElement | null)?.blur();

      fireEvent.keyDown(document.body, { key: 'Enter' });

      expect(clicks).toBe(1);
      enter.removeEventListener('click', onClick);
    });

    it('leaves Enter alone when another control has focus', () => {
      render(Hero);
      const enter = screen.getByRole('link', { name: /enter/i });
      const onClick = (event: Event) => event.preventDefault();
      enter.addEventListener('click', onClick);
      let clicks = 0;
      enter.addEventListener('click', () => clicks++);
      const other = document.createElement('button');
      document.body.appendChild(other);
      other.focus();

      fireEvent.keyDown(other, { key: 'Enter' });

      expect(clicks).toBe(0);
      enter.removeEventListener('click', onClick);
      other.remove();
    });
  });

  // test-change: R23 (the title-screen poster is hero-moon-living-poster.jpg; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T22)
  it('shows the living poster as a WebP picture with a JPEG fallback, explicit dimensions and no alt text', () => {
    const { container } = render(Hero);
    const source = container.querySelector('picture source');
    expect(source?.getAttribute('type')).toBe('image/webp');
    expect(source?.getAttribute('srcset')).toBe(
      '/art/landing/hero-moon-living-poster-960.webp 960w, /art/landing/hero-moon-living-poster-1920.webp 1920w'
    );
    const poster = container.querySelector('img.hero-poster');
    expect(poster?.getAttribute('src')).toBe(
      '/art/landing/hero-moon-living-poster.jpg'
    );
    expect(poster?.getAttribute('width')).toBe('1920');
    expect(poster?.getAttribute('height')).toBe('1080');
    expect(poster?.getAttribute('alt')).toBe('');
    expect(poster?.getAttribute('aria-hidden')).toBe('true');
    expect(poster?.getAttribute('fetchpriority')).toBe('high');
  });

  it('adds one CSS pumpkin glow over the painted pumpkin and no second familiar image (T22)', () => {
    const { container } = render(Hero);
    expect(container.querySelectorAll('.pumpkin-glow')).toHaveLength(1);
    expect(
      container.querySelector('.pumpkin-glow')?.getAttribute('aria-hidden')
    ).toBe('true');
    expect(sourcesOf(container).some((src) => src?.includes('familiar'))).toBe(
      false
    );
  });

  it('does not mount the video before first paint', () => {
    const { container } = render(Hero);
    expect(container.querySelector('video')).toBeNull();
  });

  it('defers every overlay until after first paint, so the poster is the only first-load image (T18)', async () => {
    const { container } = render(Hero, { props: { plan: STATIC } });
    expect(sourcesOf(container)).not.toContain('/art/fx/fog-layer-cold.webp');
    expect(sourcesOf(container)).not.toContain('/art/fx/embers.webp');
    await waitFor(
      () => expect(sourcesOf(container)).toContain('/art/fx/embers.webp'),
      {
        timeout: 3000
      }
    );
  });

  it('shows the fog and embers overlays in the static plan, and no smoke or bats (T16)', async () => {
    const { container } = await renderSettled(STATIC);
    const sources = sourcesOf(container);
    expect(sources).toContain('/art/fx/fog-layer-cold.webp');
    expect(sources).toContain('/art/fx/embers.webp');
    expect(sources).not.toContain('/art/fx/smoke-plume-ash.webp');
    expect(sources).not.toContain('/art/fx/bat-swarm.webp');
  });

  it('adds smoke and bats in the motion plan, hidden until GSAP sets them (T16)', async () => {
    const { container } = await renderSettled(MOTION);
    const smoke = container.querySelector('img.layer-smoke');
    const bats = container.querySelector('img.layer-bats');
    expect(smoke?.getAttribute('src')).toBe('/art/fx/smoke-plume-ash.webp');
    expect(bats?.getAttribute('src')).toBe('/art/fx/bat-swarm.webp');
    // Starts invisible so the overlays do not pop in before the scene timelines load.
    expect(smoke?.getAttribute('style')).toContain('opacity: 0');
    expect(bats?.getAttribute('style')).toContain('opacity: 0');
  });

  it('drops the far fog layer when the plan turns it off (phone rule, R26)', async () => {
    const desktop = await renderSettled(MOTION);
    expect(sourcesOf(desktop.container)).toContain(
      '/art/fx/fog-layer-cold.webp'
    );
    desktop.unmount();

    const phone: LandingPlan = {
      ...MOTION,
      layers: { ...MOTION.layers, far: false },
      bats: false
    };
    const { container } = await renderSettled(phone);
    expect(sourcesOf(container)).not.toContain('/art/fx/fog-layer-cold.webp');
  });

  it('shows the flat moon and graveyard SVGs when the poster fails to load (T16 fallback)', async () => {
    const { container } = render(Hero, { props: { plan: STATIC } });
    const poster = container.querySelector(
      'img.hero-poster'
    ) as HTMLImageElement;
    await fireEvent.error(poster);
    const sources = sourcesOf(container);
    expect(sources).toContain('/landing/moon.svg');
    expect(sources).toContain('/landing/graveyard-far.svg');
    expect(sources).toContain('/landing/graveyard-near.svg');
  });
});
