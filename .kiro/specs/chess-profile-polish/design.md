# Design: chess-profile-polish

## Sources
Issue #131; existing themed-chess-experience and profile-onboarding specs; AGENTS.md, docs/palette.md and repository steering. Existing code inspected on 25 Nov 2026.

## Chess presentation
Current asset web/static/break/pieces-steel-night.svg is 8,628 bytes, with 40×40 groups and flat fills. The shared web/src/lib/break/chess/theme.js mounts cm-chessboard for bot and puzzle. Replace artwork, not game logic. Keep wp, wn, wb, wr, wq, wk, bp, bn, bb, br, bq, bk and crown IDs, 40×40 coordinate frame and tileSize 40. Same-origin static SVG, no external resources/scripts, no WebGL or heavy dependencies. Use gradients and controlled bevels/highlights; keep filters cheap and within bounds. Aim for roughly 85–90% maximum extent where appropriate, allowing natural size hierarchy. Check actual painted bounds including strokes/shadows, not just path bounds; use safe margins. Do not stretch assets or change board palette to compensate for weak artwork. Verify both sides against the actual rendered square colors and move markers. Preserve shared mount and current board sizing unless a demonstrated regression requires a focused fix. During movement crossing a square boundary is normal; visual footprint stays at most one square.

## Profile layout
Current web/src/routes/settings/account/+page.svelte renders an 88px avatar beside identity and exposes the storage message; web/src/lib/auth/ProfileForm.svelte handles display_name/country. Mobile: stacked centered avatar, full name, @handle, readable country (omit unset), Edit profile. Desktop may use a compact adaptive layout; preserve semantic hierarchy and section order. Put email into private account details. Remove infrastructure messaging everywhere it appears in user-facing profile/onboarding. Keep avatar static/noninteractive. Avoid decorative crypt key competing with identity on narrow screens.

## Shared form and data
Reuse ProfileForm on settings and onboarding. Use “Full name” for existing display_name, not a legal identity claim. No database migration needed. Keep PATCH /api/me/profile and returned MeUser/session update. Existing Rust ProfilePatch trims name and validates length, normalizes country casing and accepts two ASCII letters. Use a maintained ISO alpha-2 country dataset and accessible searchable combobox (native fallback if appropriate); show names via consistent lookup, store code/null. Preserve unknown legacy codes until deliberately changed; don't broaden server write permissions or silently reject existing users. Avoid new dependency if a suitable existing component/data source exists. Explicit cancel discards draft, successful save updates heading and status, failure retains draft and shows safe error. Preserve capabilities, CSRF, no-store and caller-only update. No avatar endpoint.

## Correctness properties
- For each piece ID and supported viewport, settled painted bounds fit its square, aspect ratio unchanged; both routes use one sprite.
- For any valid name/country, successful save then reload yields normalized persisted values; cancel/failure leaves persisted identity unchanged.
- For demo/signed-out or another user, existing access restrictions remain; a canary account cannot be altered.
- At all specified widths, identity and controls remain readable, keyboard accessible and without horizontal overflow.

## Test and evidence strategy
Capture current baseline before changes. Use focused SVG contract tests, browser geometry and rendered screenshots, legal bot exchange and puzzle sequence, promotion/markers and reduced-motion checks. Use shared form/component tests, authenticated Playwright UI round trip, cancel/error/long-name/country keyboard cases, onboarding regression and existing auth tests. Run repository-required lint, format, type checks, Rust/web tests, build/artifact guard, security/CI and review. Record evidence in verification markdown, then after approved deployment verify live routes with permitted test account and no private data in screenshots. Owner reviews art; do not equate test success with visual approval.
