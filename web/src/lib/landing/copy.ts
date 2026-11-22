// Every visible string on the landing page. Review copy here, not in the components.
// Rule (R9): never say "smallest possible", "proven minimum" or "optimal" about a ghost input.

/** The page's only h1. The wordmark is real text, styled in CSS, never an image (title screen). */
export const WORDMARK = 'Counterspell';

/** The tagline under the wordmark (title screen). */
export const TAGLINE = 'Outthink the Machine. Sharpen Your Mind.';

/** The supporting description under the tagline on the title screen. */
export const HERO_BODY =
  'AI can give you answers. Counterspell challenges you to find your own. Solve coding challenges, think critically, and discover why your solutions fail. Build the skills to solve problems without relying on AI to do the thinking for you.';

/** The Ghost Engine section heading below the title screen. */
export const HEADLINE = "Don't Just Fix the Code. Understand the Mistake.";

/** A short paragraph under the headline that positions deliberate practice. */
export const INTRO_BODY =
  'Counterspell is deliberate practice for people who want to keep their problem-solving edge. Each challenge asks you to reason, test your assumptions, and work through failure yourself.';

/** The Ghost Engine explanation under the headline. */
export const SUPPORT_LINE =
  'Passing a test is one thing. Understanding why your solution works—or why it fails—is what makes you a better problem solver. Counterspell goes beyond telling you that your answer is wrong. Its Ghost Engine hunts for inputs that break your code, reduces them to simpler examples, and helps you identify the underlying mistake.';

/** The three visitor-facing benefits, in reading order. */
export const BENEFITS = [
  {
    heading: 'Think independently',
    body: 'Strengthen reasoning by solving problems yourself.'
  },
  {
    heading: 'Learn from failure',
    body: 'Understand why solutions break rather than simply receiving a pass/fail result.'
  },
  {
    heading: 'Build real mastery',
    body: 'Develop the ability to approach unfamiliar problems with confidence.'
  }
] as const;

export interface Beat {
  heading: 'Submit' | 'Hunt' | 'Understand';
  body: string;
}

/** The three beats that explain the ghost input, in reading order. */
export const BEATS: readonly Beat[] = [
  {
    heading: 'Submit',
    body: 'Write your solution and put your reasoning to the test.'
  },
  {
    heading: 'Hunt',
    body: 'When your solution fails, the Ghost Engine searches for a counterexample and reduces it to a simpler failing case.'
  },
  {
    heading: 'Understand',
    body: 'See what went wrong, understand why, and improve your approach.'
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
  primary: 'Start Your First Challenge'
} as const;
