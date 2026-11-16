// test-change: R23 (the title-screen loop is now hero-moon-living-*; design amendment 15 Nov 2026 19:20 SAST, landing-steel-night T22)
import { describe, expect, it } from 'vitest';
import { chooseHeroVideo } from './hero-media';

const CALM = { reducedMotion: false, viewportWidth: 1280, saveData: false, tabVisible: true };

describe('chooseHeroVideo (T15)', () => {
	it('gives no video when the visitor asks for reduced motion', () => {
		expect(chooseHeroVideo({ ...CALM, reducedMotion: true })).toBeNull();
	});

	it('gives no video when Save-Data is on', () => {
		expect(chooseHeroVideo({ ...CALM, saveData: true })).toBeNull();
	});

	it('gives no video while the tab is hidden', () => {
		expect(chooseHeroVideo({ ...CALM, tabVisible: false })).toBeNull();
	});

	it('uses the 960 px loop at 320 CSS px', () => {
		expect(chooseHeroVideo({ ...CALM, viewportWidth: 320 })).toEqual({
			webm: '/art/landing/hero-moon-living-960.webm',
			mp4: '/art/landing/hero-moon-living-960.mp4'
		});
	});

	it('uses the 960 px loop at exactly 900 CSS px', () => {
		expect(chooseHeroVideo({ ...CALM, viewportWidth: 900 })?.webm).toBe(
			'/art/landing/hero-moon-living-960.webm'
		);
	});

	it('uses the 1920 px loop just above 900 CSS px', () => {
		expect(chooseHeroVideo({ ...CALM, viewportWidth: 901 })).toEqual({
			webm: '/art/landing/hero-moon-living-1920.webm',
			mp4: '/art/landing/hero-moon-living-1920.mp4'
		});
	});

	it('uses the 1920 px loop on a 1280 CSS px desktop', () => {
		expect(chooseHeroVideo(CALM)?.mp4).toBe('/art/landing/hero-moon-living-1920.mp4');
	});
});
