// Every visible string on the landing page. Review copy here, not in the components.
// Rule (R9): never say "smallest possible", "proven minimum" or "optimal" about a ghost input.

/** The page's only h1. The wordmark is real text, styled in CSS, never an image (title screen). */
export const WORDMARK = 'Counterspell';

/** The tagline under the wordmark (title screen). */
export const TAGLINE = 'Master the algorithms that matter';

/** The page headline, now shown below the title screen as the section heading. */
export const HEADLINE = 'Learn why your code breaks.';

/** A short paragraph under the headline that says what the page does. */
export const INTRO_BODY =
	'Most practice sites only tell you that a test failed. Counterspell shows you the input that makes your code fail, so you can see the mistake and fix the idea behind it.';

/** The line under the headline. */
export const SUPPORT_LINE =
	'Your solution fails. Counterspell hunts down the input that breaks it, shrinks it to the smallest failing input it can find, and explains the mistake.';

export interface Beat {
	heading: 'Submit' | 'Hunt' | 'Understand';
	body: string;
}

/** The three beats that explain the ghost input, in reading order. */
export const BEATS: readonly Beat[] = [
	{
		heading: 'Submit',
		body: 'Your code runs in a sealed sandbox on our server. A trusted judge holds the tests and the answers, so your program only prints what it computes.'
	},
	{
		heading: 'Hunt',
		body: 'When the code is wrong, the ghost engine searches for an input that breaks it, then shrinks that input. You get the smallest failing input it can find within its time budget.'
	},
	{
		heading: 'Understand',
		body: 'You see the failing input, what you expected, what you got and the mistake behind it, so you learn the concept and not just the fix.'
	}
];

/** Fixed values for the illustrative ghost example. Nothing here is computed or fetched. */
export const EXAMPLE = {
	input: 'nums = [0, 0], target = 0',
	expected: '[0, 1]',
	got: '[0, 0]'
} as const;

/** Caption shown under the example so it is never read as a real run. */
export const CAPTION = 'Illustration, not a real run.';

/** Call to action labels. */
export const CTA = {
	primary: 'Start practising',
	secondary: 'Log in'
} as const;
