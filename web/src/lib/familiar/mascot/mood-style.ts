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
	/** The artwork layer (an `<image>` in the contract SVG) that shows this mood's painting. */
	layer: string;
	icon: MoodIcon;
	glow: GlowRecipe;
}

const STYLES: Record<Mood, MoodStyle> = {
	1: { name: 'furious', layer: 'fam-mood-1', icon: 'cross', glow: { base: '--danger-fill' } },
	2: {
		name: 'angry',
		layer: 'fam-mood-2',
		icon: 'warning',
		glow: { base: '--danger-fill', mixWith: '--accent-candle', mixPercent: 55 }
	},
	3: {
		name: 'uneasy',
		layer: 'fam-mood-3',
		icon: 'caution',
		glow: { base: '--accent-candle', mixWith: '--bg', mixPercent: 55 }
	},
	4: { name: 'pleased', layer: 'fam-mood-4', icon: 'smile', glow: { base: '--accent-candle' } },
	5: {
		name: 'laughing',
		layer: 'fam-mood-5',
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

/** The CSS colour for a glow recipe. It names palette tokens only, so no hex value is written. */
export function glowCss(recipe: GlowRecipe): string {
	if (recipe.mixWith === undefined || recipe.mixPercent === undefined) {
		return `var(${recipe.base})`;
	}
	return `color-mix(in srgb, var(${recipe.base}) ${recipe.mixPercent}%, var(${recipe.mixWith}))`;
}
