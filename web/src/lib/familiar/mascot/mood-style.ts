// Mood styling for the mascot. Pure: no DOM, no colours. Glows name palette tokens and a mix
// percentage. The browser reads the token values at runtime, so no hex value lives here.
import type { Mood } from '../types';
import { REQUIRED_IDS } from './contract';

/** A palette token name, for example `--accent-candle`. */
export type TokenName = string;

/** The icon shown with the mood, so colour is never the only signal. */
export type MoodIcon = 'cross' | 'warning' | 'caution' | 'smile' | 'sparkle';

/** How the glow is built: the base token, optionally mixed with another token. */
export interface GlowRecipe {
	base: TokenName;
	mixWith?: TokenName;
	mixPercent?: number;
}

/** Everything the view needs to draw one mood. */
export interface MoodStyle {
	name: 'furious' | 'angry' | 'uneasy' | 'pleased' | 'laughing';
	eyesLayer: string;
	mouthLayer: string;
	icon: MoodIcon;
	glow: GlowRecipe;
}

const STYLES: Record<Mood, MoodStyle> = {
	1: {
		name: 'furious',
		eyesLayer: 'fam-eyes-1',
		mouthLayer: 'fam-mouth-1',
		icon: 'cross',
		glow: { base: '--danger-fill' }
	},
	2: {
		name: 'angry',
		eyesLayer: 'fam-eyes-2',
		mouthLayer: 'fam-mouth-2',
		icon: 'warning',
		glow: { base: '--danger-fill', mixWith: '--accent-candle', mixPercent: 55 }
	},
	3: {
		name: 'uneasy',
		eyesLayer: 'fam-eyes-3',
		mouthLayer: 'fam-mouth-3',
		icon: 'caution',
		glow: { base: '--accent-candle', mixWith: '--bg', mixPercent: 55 }
	},
	4: {
		name: 'pleased',
		eyesLayer: 'fam-eyes-4',
		mouthLayer: 'fam-mouth-4',
		icon: 'smile',
		glow: { base: '--accent-candle' }
	},
	5: {
		name: 'laughing',
		eyesLayer: 'fam-eyes-5',
		mouthLayer: 'fam-mouth-5',
		icon: 'sparkle',
		glow: { base: '--accent-candle-glow' }
	}
};

/** The style for a mood. Every mood from 1 to 5 has one. */
export function moodStyle(mood: Mood): MoodStyle {
	return STYLES[mood];
}

/**
 * The text colour token for the mood text. Failed runs use the danger text colour; passing runs
 * never do, even for the worst complexity tier.
 */
export function textTokenFor(failed: boolean): TokenName {
	return failed ? '--danger' : '--text';
}

/** Every layer id a mood can draw, so the view never binds to an id the artwork lacks. */
export const LAYER_IDS: readonly string[] = REQUIRED_IDS;
