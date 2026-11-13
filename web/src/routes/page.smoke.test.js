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
    expect(
      screen.getByRole('heading', { name: /learn why your code breaks/i })
    ).toBeTruthy();
  });
});
