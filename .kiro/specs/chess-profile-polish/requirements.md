# Requirements: chess-profile-polish

## Purpose
Improve chess piece readability and player identity UX. Requirements-first follow-up to themed-chess-experience and profile-onboarding. Tracking issue: #131. Spec only; implementation pending.

## Fixed decisions
- Preserve Steel Night, shared SVG, chess rules, account security and existing profile persistence.
- Supersede profile-onboarding PO-3.5's storage-unavailable copy and PO-3.1's identity email placement. All other security/onboarding requirements remain.
- No avatar storage/upload, handle editing, legal-name verification, game engine changes or unrelated redesign.

## CPX-1: Chess artwork
As a player, I want pieces that pop on both square colors without crowding the board.
1. THE sprite SHALL provide original, coherent white and black designs for all six piece types and the promotion crown, with recognizable silhouettes and premium dimensional shading.
2. THE artwork SHALL use carved ivory and restrained candle-gold for white, obsidian with bright neutral/steel edging for black, compatible with Steel Night.
3. WHEN displayed on either square color or selected/last-move states, pieces SHALL remain visually distinguishable by type and side.
4. WHEN a viewport changes, pieces SHALL scale proportionally to their square. Settled artwork, including outlines and shadows, SHALL remain fully contained with visible padding. During animation/dragging a piece may traverse squares but SHALL retain single-square dimensions and remain unclipped.
5. THE bot and puzzle pages SHALL share the sprite and preserve rules, orientation, promotion, input, accessible naming, reduced motion and score persistence.

## CPX-2: Profile identity
As a learner, I want a clean profile that shows my name before my username.
1. WHEN on mobile, THE avatar SHALL be circular and centered above full/display name, distinct @handle, optional readable country and Edit profile action when permitted.
2. THE name SHALL be the primary heading; absent names SHALL have a sensible fallback without repeating the handle. Email SHALL appear only in private account details, once.
3. THE interface SHALL NOT display internal storage/configuration messages or a nonfunctional upload control; retain existing avatar/fallback.
4. THE layout SHALL adapt to desktop and narrow screens without overflow or cramped decorative artwork, preserving account security section order.

## CPX-3: Profile editing
As a learner, I want to edit my identity without knowing internal codes.
1. THE shared form SHALL label display_name “Full name”, accept a chosen name of 1–80 characters after trimming (including Unicode, punctuation and spaces), and keep the handle unchanged.
2. THE form SHALL offer an accessible searchable country selector with readable names, optional clearing and ISO alpha-2 values; legacy unknown values SHALL not silently change.
3. WHEN saving, THE form SHALL prefill current values, prevent duplicate requests, show meaningful success/error states and update the shared session only after a successful response.
4. WHEN cancelled, unsaved changes SHALL NOT persist. On reload/relogin, saved values SHALL remain visible.
5. THE existing authenticated PATCH, CSRF, user isolation, demo restriction and onboarding completion/return behavior SHALL remain intact.

## CPX-4: Verification
1. THE agent SHALL capture before/after rendered evidence of all pieces on both colors and both chess routes at 320/390px and 1440px, plus profile at 320/390/768/1440px including editing.
2. THE agent SHALL add regression tests for sprite contract/bounds, gameplay and profile save/cancel/error/keyboard flows, including reload and isolation.
3. THE agent SHALL follow repository TDD, review, CI and deployment requirements. Actual visual approval and live UI verification are completion gates; no claim of implementation from this spec alone.
