import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import { goto } from '$app/navigation';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Page from './+page.svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

describe('landing route composition (T08)', () => {
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.mocked(goto).mockClear();
  });

  it('keeps the smoke marker on the root main element with lang en', () => {
    const { container } = render(Page);
    const main = container.querySelector('main');
    expect(main?.getAttribute('data-smoke-marker')).toBe(
      'counterspell-svelte-static'
    );
    expect(main?.getAttribute('lang')).toBe('en');
  });

  it('shows the wordmark h1, the three beat headings in order and the sign-up link', () => {
    const { container } = render(Page);
    // test-change: R20 (the title-screen wordmark is the page h1; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T23)
    expect(screen.getByRole('heading', { level: 1 }).textContent?.trim()).toBe(
      'Counterspell'
    );
    // test-change: R21 (the headline is the section h2 and the beats are h3; landing-steel-night T25)
    expect(
      screen.getByRole('heading', {
        level: 2,
        name: "Don't Just Fix the Code. Understand the Mistake."
      })
    ).toBeTruthy();
    const beats = Array.from(container.querySelectorAll('section h3')).map(
      (h) => h.textContent?.trim()
    );
    expect(beats).toEqual(['Submit', 'Hunt', 'Understand']);
    expect(container.textContent).toContain('Think independently');
    expect(container.textContent).toContain('Learn from failure');
    expect(container.textContent).toContain('Build real mastery');
    expect(
      screen
        .getByRole('link', { name: 'Start Your First Challenge' })
        .getAttribute('href')
    ).toBe('/problems');
  });

  it('preloads the self-hosted Creepster woff2 in the document head (T21)', () => {
    render(Page);
    const preload = document.head.querySelector(
      'link[rel="preload"][as="font"]'
    );
    expect(preload, 'font preload link').not.toBeNull();
    expect(preload?.getAttribute('type')).toBe('font/woff2');
    expect(preload?.getAttribute('crossorigin')).not.toBeNull();
    expect(preload?.getAttribute('href')).toMatch(/Creepster.*\.woff2/);
    expect(preload?.getAttribute('href')).not.toMatch(/^https?:/);
  });

  // The poster is the largest contentful paint and the browser must pick the compact AVIF candidate.
  it('uses the responsive AVIF poster candidates (T13)', () => {
    const { container } = render(Page);
    const source = container.querySelector('picture source');
    expect(source, 'hero picture source').not.toBeNull();
    expect(source?.getAttribute('type')).toBe('image/avif');
    expect(source?.getAttribute('srcset')).toContain(
      'hero-moon-living-poster-1464.avif 1464w'
    );
    expect(source?.getAttribute('srcset')).not.toMatch(/https?:/);
  });

  it('sends a signed-in visitor to the dashboard without a 401 probe', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        Promise.resolve(
          new Response(JSON.stringify({ signed_in: true }), {
            status: 200,
            headers: { 'content-type': 'application/json' }
          })
        )
      )
    );

    render(Page);

    await waitFor(() => expect(goto).toHaveBeenCalledWith('/dashboard'));
    expect(fetch).toHaveBeenCalledWith(
      '/api/auth/session',
      expect.objectContaining({ credentials: 'same-origin' })
    );
  });

  it('uses the requested SEO title and description', () => {
    render(Page);
    expect(document.title).toBe(
      'Counterspell — Sharpen Your Mind with Coding Challenges'
    );
    expect(
      document.head
        .querySelector('meta[name="description"]')
        ?.getAttribute('content')
    ).toContain('stronger independent problem solver');
  });

  it('no longer shows the skeleton milestone message', () => {
    const { container } = render(Page);
    expect(container.textContent).not.toContain('Skeleton milestone');
  });
});
