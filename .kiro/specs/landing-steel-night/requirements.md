# Requirements: landing-steel-night

## Purpose

Build the public landing page at `/` in the Steel Night look: a cinematic moon-and-graveyard hero, scroll reveals, a three-beat explanation of the ghost input, and a sign-up call to action. The page is the first thing a judge sees, so it must load fast on a phone, work without motion, and make the ghost idea clear in under a minute.

This is a Quick Spec for a visuals-first milestone (Fri 13 Nov target). It is static web work only. It adds no server endpoint, no Rust code, no database table and no real run result.

Sources: `docs/system-design.md` §0, §1.2, §16, §14 (landing polish is never cut); `docs/requirements.md` §7 (Lighthouse, accessibility, reduced motion); `docs/palette.md` (tokens, type, motion rules); `docs/product-brief.md` (look and feel); `docs/licence-rules.md` (GSAP exception, OFL fonts); steering `frontend.md`, `product.md`, `release-checklist.md`; sibling specs `repo-skeleton` (static page, smoke marker, prerender), `ide-workspace` (font loading and tokens), `auth` (sign-up route), `efficiency-familiar` (asset and GSAP conventions).

## Scope constraints

- The landing page SHALL be part of the existing `web/` SvelteKit app and SHALL be prerendered as static HTML by the existing adapter-static setup.
- GSAP SHALL be used only on the landing page and in the Familiar. The IDE and dashboard SHALL NOT import it.
- The page SHALL NOT present fabricated judging results, run times, verdicts or user counts. Any example ghost on the page SHALL be labelled as an illustration.
- The page SHALL NOT use a serif typeface, purple, or any colour outside the Steel Night tokens.
- The page SHALL NOT use raw hex values in components. Colour comes from semantic tokens in `web/src/app.css`.
- Dark mode is the only theme. The page SHALL NOT switch to a light theme based on the operating system setting.
- No new Cargo package, no new server route and no change to `counterspell-server` are in scope.
- The existing skeleton smoke-test marker (`data-smoke-marker="counterspell-svelte-static"`) SHALL remain on the root element of `/`, so the Compose smoke test keeps working.

## EARS acceptance criteria

### Route and rendering

1. WHEN a visitor requests `/`, THE SYSTEM SHALL serve the landing page as prerendered static HTML through the existing Rust static route, with no development server involved.
2. THE SYSTEM SHALL keep the root element of `/` carrying `data-smoke-marker="counterspell-svelte-static"`.
3. THE SYSTEM SHALL render the headline, the three ghost beats and the sign-up call to action in the static HTML, so the content is readable before any JavaScript runs.
4. IF JavaScript is disabled or fails to load, THEN THE SYSTEM SHALL still show all text, beats and the call to action, with no content hidden behind a scroll trigger.

### Hero

5. THE SYSTEM SHALL show a hero with a moon and a graveyard silhouette, built from separate SVG files in `web/static/landing/`, not from a single composite image.
6. THE SYSTEM SHALL give every decorative SVG `aria-hidden="true"` and `focusable="false"`, and SHALL give the hero headline the text "Learn why your code breaks." as the page's `h1`.
7. THE SYSTEM SHALL give each SVG explicit `width` and `height` attributes so the layout does not shift while they load.

### Ghost-input explanation

8. THE SYSTEM SHALL explain the ghost input in exactly three beats, in this order: **Submit** (the code is judged in a sealed sandbox on our server), **Hunt** (when the code is wrong, the engine searches for an input that breaks it and shrinks it), and **Understand** (the input, the expected and actual output, and the mistake behind it are shown).
9. THE SYSTEM SHALL describe the Hunt result as the smallest failing input *found* within the time budget, and SHALL NOT describe it as the smallest possible input, a proven minimum, or the optimal input.
10. THE SYSTEM SHALL present any example ghost on the page with its expected output and its actual output, both as plain text, and SHALL label the example as an illustration.

### Scroll reveals (GSAP)

11. WHERE motion is allowed, THE SYSTEM SHALL reveal the beats, the moon and the graveyard layers with GSAP ScrollTrigger, using transform and opacity only.
12. WHILE the page is scrolled, THE SYSTEM SHALL move the far and near graveyard layers at different rates (parallax) only when motion is allowed.
13. THE SYSTEM SHALL play each reveal once. It SHALL NOT loop any landing animation indefinitely.
14. THE SYSTEM SHALL load GSAP with a dynamic import after the first paint, and SHALL NOT import GSAP at all when reduced motion is requested.

### Reduced motion

15. WHEN `prefers-reduced-motion: reduce` is set, THE SYSTEM SHALL show the static final frame of every scene, with no ScrollTrigger, no parallax, no bat flight and no transition longer than 0 ms.
16. WHEN reduced motion is set, THE SYSTEM SHALL keep every control and beat usable and visible, with the same text as the motion version.

### Sign-up call to action

17. THE SYSTEM SHALL show one primary call to action to sign up, styled with the candle accent token, linking to the sign-up route owned by the `auth` spec.
18. THE SYSTEM SHALL show a secondary text link to the sign-in route. It SHALL NOT offer payment or any paid tier.
19. WHILE the call to action has keyboard focus, THE SYSTEM SHALL show the 2px candle focus ring with a 2px offset.

### Typography and tokens

20. THE SYSTEM SHALL set headings and body text in Geist, and SHALL set any code sample in JetBrains Mono, both self-hosted from the OFL-1.1 packages named in the design.
21. THE SYSTEM SHALL use bone text (`--text`) on the background and surfaces for all body copy, and SHALL keep every text colour pair at WCAG AA contrast or better as listed in `docs/palette.md`.
22. THE SYSTEM SHALL use bright green only for pass or success states, and red only for failures or errors. The landing page MAY show a pass example in green and a fail example in red, labelled as illustrations.

### Performance

23. THE SYSTEM SHALL score at least 90 for performance in a Lighthouse mobile run of `/`, measured as the median of three runs against the production static build served by `counterspell-server` on localhost.
24. THE SYSTEM SHALL keep the critical rendering path free of GSAP, and SHALL keep the landing route's JavaScript before GSAP within the byte budget in the design.
25. THE SYSTEM SHALL use the headline text as the largest contentful paint element, and SHALL NOT use a raster image as the largest element.
26. WHERE the viewport is narrower than 640 CSS px, THE SYSTEM SHALL render the far graveyard layer and the bat flight as static, to limit work on phones.

### Accessibility and privacy

27. THE SYSTEM SHALL keep a logical heading order (one `h1`, then `h2` for each beat section) and a page `lang="en"`.
28. THE SYSTEM SHALL NOT convey any status by colour alone; any colour-coded example SHALL also carry a text label.
29. THE SYSTEM SHALL load no third-party scripts, analytics or fonts from a CDN.

## Out of scope

- Sign-up, sign-in and account forms (owned by `auth`).
- A pricing page, payment, or a paid tier.
- Live problem previews, real run results, or the IDE.
- A light theme.
- A realistic painted art set. The art here is flat vector silhouettes; see the open questions in `design.md`.
- Any server change, any Rust code, or any database table.
