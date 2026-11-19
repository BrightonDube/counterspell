import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import ErrorView from './ErrorView.svelte';

describe('ErrorView (T20)', () => {
  afterEach(() => cleanup());

  it('explains a missing page in plain words and links back to the start', () => {
    render(ErrorView, { props: { status: 404 } });
    expect(
      screen.getByRole('heading', { level: 1, name: 'Page not found' })
    ).toBeTruthy();
    expect(
      screen.getByText(
        'We could not find that page. The link may be wrong, or the page may have moved.'
      )
    ).toBeTruthy();
    expect(
      screen
        .getByRole('link', { name: 'Back to the start' })
        .getAttribute('href')
    ).toBe('/');
  });

  it('tells the visitor the problem is on our side for a server error', () => {
    render(ErrorView, { props: { status: 500 } });
    expect(
      screen.getByRole('heading', { level: 1, name: 'Something went wrong' })
    ).toBeTruthy();
    expect(
      screen.getByText('The problem is on our side. Try again in a moment.')
    ).toBeTruthy();
  });

  it('uses the still pumpkin, hidden from assistive technology, with explicit size', () => {
    const { container } = render(ErrorView, { props: { status: 404 } });
    const art = container.querySelector('img');
    expect(art?.getAttribute('src')).toBe(
      '/art/familiar/jack-o-lantern-orange.webp'
    );
    expect(art?.getAttribute('alt')).toBe('');
    expect(art?.getAttribute('aria-hidden')).toBe('true');
    expect(art?.getAttribute('width')).toBe('256');
    expect(art?.getAttribute('height')).toBe('256');
  });

  it('shows the status code as plain text, not colour alone', () => {
    const { container } = render(ErrorView, { props: { status: 404 } });
    expect(container.textContent).toContain('404');
  });
});
