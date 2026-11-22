import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import AppShell from './AppShell.svelte';

vi.mock('$app/state', () => ({
  page: { url: new URL('https://counterspel.online/dashboard') }
}));

describe('AppShell', () => {
  afterEach(() => cleanup());

  // Breaks if the costume mark is missing, stops being decorative, or replaces the real nav.
  it('shows a decorative skull beside the wordmark without changing navigation', () => {
    const { container } = render(AppShell);

    const home = screen.getByRole('link', { name: 'Counterspell' });
    expect(home.getAttribute('href')).toBe('/dashboard');
    expect(home.querySelector('img')).toBeNull();

    const skull = container.querySelector<HTMLImageElement>(
      'img[data-testid="nav-creature"]'
    );
    expect(skull).not.toBeNull();
    expect(skull?.getAttribute('src')).toBe(
      '/art/creatures/skull-green-eyes.webp'
    );
    expect(skull?.getAttribute('alt')).toBe('');
    expect(skull?.getAttribute('width')).toBe('32');
    expect(skull?.getAttribute('height')).toBe('32');
    expect(skull?.getAttribute('aria-hidden')).toBe('true');

    for (const name of ['Problems', 'Break room', 'Settings']) {
      expect(screen.getByRole('link', { name })).toBeTruthy();
    }
  });
});
