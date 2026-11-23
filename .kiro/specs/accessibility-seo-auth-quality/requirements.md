# Accessibility, SEO and Authenticated Quality Requirements

## Goal

Make public, signed-out and authenticated routes understandable to people, assistive technology, search engines and browser diagnostics.

## Requirements

- Public routes have distinct titles, descriptions and canonical URLs.
- Shareable public routes include Open Graph and Twitter metadata.
- Problem detail has one descriptive `h1`.
- Chessboard SVGs have accessible names without duplicating move announcements.
- Back controls, checkboxes and other interactive targets meet the 24 by 24 CSS-pixel minimum.
- Dashboard landmarks are unique.
- Disabled onboarding controls meet contrast requirements.
- Signed-out chess copy must not imply that already signed-in users are anonymous.
- Expected anonymous states must not emit noisy expected-401 console errors.
- Session and profile data should be reused so account and onboarding do not issue avoidable duplicate requests.
- Puzzle layout shift and other unstable surfaces must be reduced.

## Acceptance

Affected routes pass axe with no verified violations, keyboard checks pass, metadata is visible in generated HTML and Lighthouse regressions are investigated rather than waived.
