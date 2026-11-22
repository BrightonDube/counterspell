// Chooses the hero's looping video source. Pure: no DOM, no window, no GSAP.
// The page reads the visitor's settings and passes them in.

/** Viewports up to this width (CSS px) get the 960 px loop, to save bytes on phones. */
const NARROW_MAX_WIDTH = 900;

/** The hero poster's WebP candidates. The `<picture>` and the head preload must use the same values. */
export const HERO_POSTER_SRCSET =
  '/art/landing/hero-moon-living-poster-960.avif 960w, /art/landing/hero-moon-living-poster-1200.avif 1200w, /art/landing/hero-moon-living-poster-1440.avif 1440w, /art/landing/hero-moon-living-poster-1464.avif 1464w, /art/landing/hero-moon-living-poster-1680.avif 1680w';

/** WebP candidates for browsers that cannot decode AVIF. */
export const HERO_POSTER_WEBP_SRCSET =
  '/art/landing/hero-moon-living-poster-960.webp 960w, /art/landing/hero-moon-living-poster-1920.webp 1920w';

/** The hero plate is 16:9 and covers the viewport, so width follows viewport height. */
export const HERO_POSTER_SIZES = 'calc(100vh * 1.7778)';

export interface HeroVideoFacts {
  /** True when the visitor has asked for reduced motion. */
  reducedMotion: boolean;
  /** Viewport width in CSS pixels. */
  viewportWidth: number;
  /** True when the browser reports Save-Data (the visitor wants less data). */
  saveData: boolean;
  /** False while the tab is hidden, so no video plays in the background. */
  tabVisible: boolean;
}

export interface HeroVideo {
  /** WebM source, tried first. */
  webm: string;
  /** MP4 source, used by browsers that cannot play WebM. */
  mp4: string;
}

/**
 * Returns the video sources for the hero, or null when no video should load.
 * Reduced motion, Save-Data and a hidden tab all mean no video at all; the poster stays.
 */
export function chooseHeroVideo(facts: HeroVideoFacts): HeroVideo | null {
  if (facts.reducedMotion || facts.saveData || !facts.tabVisible) {
    return null;
  }
  const width = facts.viewportWidth <= NARROW_MAX_WIDTH ? 960 : 1920;
  return {
    webm: `/art/landing/hero-moon-living-${width}.webm`,
    mp4: `/art/landing/hero-moon-living-${width}.mp4`
  };
}
