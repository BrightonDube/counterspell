import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import SignUpCta from './SignUpCta.svelte';

describe('SignUpCta', () => {
  afterEach(() => cleanup());

  it('links the primary call to action to the problems catalogue', () => {
    render(SignUpCta);
    const primary = screen.getByRole('link', { name: 'Start practising' });
    expect(primary.getAttribute('href')).toBe('/problems');
  });

  it('offers one call to action — no duplicate sign-in links', () => {
    render(SignUpCta);
    expect(screen.getAllByRole('link')).toHaveLength(1);
  });

  it('styles the primary button with the candle accent and a visible focus ring', () => {
    render(SignUpCta);
    const primary = screen.getByRole('link', { name: 'Start practising' });
    expect(primary.className).toContain('bg-accent-candle');
    expect(primary.className).toContain('hover:bg-accent-candle-glow');
    expect(primary.className).toContain('focus-visible:ring-2');
    expect(primary.className).toContain('focus-visible:ring-offset-2');
    expect(primary.className).toContain('focus-visible:ring-focus');
  });

  it('places a decorative candle beside the call to action, hidden from assistive technology (T17)', () => {
    const { container } = render(SignUpCta);
    const art = container.querySelector('img');
    expect(art?.getAttribute('src')).toBe('/art/objects/candle-flame.webp');
    expect(art?.getAttribute('alt')).toBe('');
    expect(art?.getAttribute('aria-hidden')).toBe('true');
    expect(art?.getAttribute('width')).toBe('96');
    expect(art?.getAttribute('height')).toBe('96');
    expect(art?.getAttribute('loading')).toBe('lazy');
  });

  it('places the orange jack-o-lantern still beside the call to action as a lazy, decorative WebP (T19)', () => {
    const { container } = render(SignUpCta);
    const pumpkin = container.querySelector(
      'img[src="/art/familiar/jack-o-lantern-orange.webp"]'
    );
    expect(pumpkin).not.toBeNull();
    expect(pumpkin?.getAttribute('alt')).toBe('');
    expect(pumpkin?.getAttribute('aria-hidden')).toBe('true');
    expect(pumpkin?.getAttribute('width')).toBe('96');
    expect(pumpkin?.getAttribute('height')).toBe('96');
    expect(pumpkin?.getAttribute('loading')).toBe('lazy');
    expect(pumpkin?.getAttribute('decoding')).toBe('async');
  });

  // R24 (landing performance): a 96 px picture must not download the 512 px file.
  it('offers a 192 px candidate for each 96 px picture and keeps the 512 px file as the fallback', () => {
    const { container } = render(SignUpCta);
    const candle = container.querySelector(
      'img[src="/art/objects/candle-flame.webp"]'
    );
    const pumpkin = container.querySelector(
      'img[src="/art/familiar/jack-o-lantern-orange.webp"]'
    );
    expect(candle?.getAttribute('srcset')).toBe(
      '/art/objects/candle-flame-192.webp 192w, /art/objects/candle-flame.webp 512w'
    );
    expect(pumpkin?.getAttribute('srcset')).toBe(
      '/art/familiar/jack-o-lantern-orange-192.webp 192w, /art/familiar/jack-o-lantern-orange.webp 512w'
    );
    expect(candle?.getAttribute('sizes')).toBe('96px');
    expect(pumpkin?.getAttribute('sizes')).toBe('96px');
  });

  it('does not load the animated pumpkin SVG, so reduced motion and Save-Data get the still (T19)', () => {
    const { container } = render(SignUpCta);
    const sources = Array.from(container.querySelectorAll('img')).map((img) =>
      img.getAttribute('src')
    );
    expect(sources.some((src) => src?.endsWith('.svg'))).toBe(false);
  });

  it('offers no payment or paid tier wording', () => {
    const { container } = render(SignUpCta);
    expect(container.textContent?.toLowerCase()).not.toMatch(
      /price|pricing|paid|payment|subscri/
    );
  });
});
