import { cleanup, render, within } from '@testing-library/svelte';
import { afterEach } from 'vitest';
import { describe, expect, it } from 'vitest';
import GhostBeats from './GhostBeats.svelte';
import GhostExample from './GhostExample.svelte';

describe('GhostBeats', () => {
  // test-change: R21 (the beat headings are h3 under the new h2 section heading; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T25)
  it('renders three sections with h3 headings Submit, Hunt and Understand in order', () => {
    const { container } = render(GhostBeats);
    const headings = Array.from(container.querySelectorAll('section h3')).map(
      (h) => h.textContent?.trim()
    );
    expect(headings).toEqual(['Submit', 'Hunt', 'Understand']);
  });

  // test-change: R6 (beat icons are now approved art from web/static/art/objects, still hidden from assistive technology)
  it('gives each beat a decorative image from the approved art, hidden from assistive technology', () => {
    const { container } = render(GhostBeats);
    const sections = Array.from(container.querySelectorAll('section'));
    expect(
      sections.map((s) => s.querySelector('img')?.getAttribute('src'))
    ).toEqual([
      '/art/objects/crypt-key.webp',
      '/art/objects/crystal-ball.webp',
      '/art/objects/brain-jar.webp'
    ]);
    for (const section of sections) {
      const art = section.querySelector('img');
      expect(art?.getAttribute('alt')).toBe('');
      expect(art?.getAttribute('aria-hidden')).toBe('true');
      expect(art?.getAttribute('width')).toBe('64');
      expect(art?.getAttribute('height')).toBe('64');
      expect(art?.getAttribute('loading')).toBe('lazy');
    }
  });

  // R24 (landing performance): a 64 px icon must not download the 512 px file.
  it('offers a 128 px candidate for each 64 px beat icon and keeps the 512 px file as the fallback', () => {
    const { container } = render(GhostBeats);
    const images = Array.from(container.querySelectorAll('section img'));
    expect(images.map((img) => img.getAttribute('srcset'))).toEqual([
      '/art/objects/crypt-key-128.webp 128w, /art/objects/crypt-key.webp 512w',
      '/art/objects/crystal-ball-128.webp 128w, /art/objects/crystal-ball.webp 512w',
      '/art/objects/brain-jar-128.webp 128w, /art/objects/brain-jar.webp 512w'
    ]);
    for (const img of images) {
      expect(img.getAttribute('sizes')).toBe('64px');
    }
  });

  it('places the labelled example inside the Hunt beat', () => {
    const { container } = render(GhostBeats);
    const hunt = Array.from(container.querySelectorAll('section')).find((s) =>
      s.querySelector('h3')?.textContent?.includes('Hunt')
    );
    expect(hunt?.textContent).toContain('Illustration, not a real run.');
  });
});

describe('GhostExample', () => {
  afterEach(() => cleanup());

  it('labels the expected and got values with text, not colour alone', () => {
    const { container } = render(GhostExample);
    const view = within(container);
    expect(view.getByText('expected')).toBeTruthy();
    expect(view.getByText('got')).toBeTruthy();
    expect(view.getByText('[0, 1]')).toBeTruthy();
    expect(view.getByText('[0, 0]')).toBeTruthy();
  });

  it('uses the success token for expected and the danger token for got', () => {
    const { container } = render(GhostExample);
    const view = within(container);
    const expectedValue = view.getByText('[0, 1]');
    const gotValue = view.getByText('[0, 0]');
    expect(expectedValue.className).toContain('text-success');
    expect(gotValue.className).toContain('text-danger');
    expect(container.textContent).toContain('nums = [0, 0], target = 0');
  });

  it('states that the example is an illustration', () => {
    const { container } = render(GhostExample);
    expect(
      within(container).getByText('Illustration, not a real run.')
    ).toBeTruthy();
  });
});
