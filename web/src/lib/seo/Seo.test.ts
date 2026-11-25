// AQ01: shared route metadata. Every public route renders a distinct title,
// description, canonical URL, Open Graph and Twitter card via <Seo>.
import { cleanup, render } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import Seo from './Seo.svelte';

const head = () => document.head;
const meta = (selector: string) =>
  head().querySelector(selector)?.getAttribute('content');

describe('Seo', () => {
  beforeEach(() => {
    head()
      .querySelectorAll(
        'title, meta[name="description"], link[rel="canonical"], meta[property^="og:"], meta[name^="twitter:"]'
      )
      .forEach((el) => el.remove());
  });
  afterEach(cleanup);

  it('emits title, description and the prod canonical URL', () => {
    render(Seo, {
      title: 'Problems · Counterspell',
      description: 'Browse the catalogue.',
      path: '/problems'
    });
    expect(document.title).toBe('Problems · Counterspell');
    expect(meta('meta[name="description"]')).toBe('Browse the catalogue.');
    expect(
      head().querySelector('link[rel="canonical"]')?.getAttribute('href')
    ).toBe('https://counterspel.online/problems');
  });

  it('emits Open Graph and Twitter metadata for shareable routes', () => {
    render(Seo, {
      title: 'Counterspell',
      description: 'Sharpen your mind.',
      path: '/'
    });
    expect(meta('meta[property="og:title"]')).toBe('Counterspell');
    expect(meta('meta[property="og:description"]')).toBe('Sharpen your mind.');
    expect(meta('meta[property="og:url"]')).toBe('https://counterspel.online/');
    expect(meta('meta[property="og:type"]')).toBe('website');
    expect(meta('meta[property="og:site_name"]')).toBe('Counterspell');
    expect(meta('meta[property="og:image"]')).toMatch(
      /^https:\/\/counterspel\.online\//
    );
    expect(meta('meta[name="twitter:card"]')).toBe('summary_large_image');
    expect(meta('meta[name="twitter:title"]')).toBe('Counterspell');
    expect(meta('meta[name="twitter:description"]')).toBe('Sharpen your mind.');
    expect(meta('meta[name="twitter:image"]')).toMatch(
      /^https:\/\/counterspel\.online\//
    );
  });

  it('lets a route opt out of social metadata (private screens)', () => {
    render(Seo, {
      title: 'Dashboard · Counterspell',
      description: 'Your practice.',
      path: '/dashboard',
      social: false
    });
    expect(document.title).toBe('Dashboard · Counterspell');
    expect(head().querySelector('meta[property="og:title"]')).toBeNull();
    expect(head().querySelector('meta[name="twitter:card"]')).toBeNull();
    expect(
      head().querySelector('link[rel="canonical"]')?.getAttribute('href')
    ).toBe('https://counterspel.online/dashboard');
  });
});
