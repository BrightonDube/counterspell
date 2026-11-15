import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import Page from './+page.svelte';

describe('landing page', () => {
  it('renders the Counterspell heading and smoke marker', () => {
    const { container } = render(Page);
    expect(
      container.querySelector(
        '[data-smoke-marker="counterspell-svelte-static"]'
      )
    ).toBeTruthy();
    // test-change: R20 (the wordmark is the page h1; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T23)
    expect(
      screen.getByRole('heading', { name: /^counterspell$/i, level: 1 })
    ).toBeTruthy();
  });
});
