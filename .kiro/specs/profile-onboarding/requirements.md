# Requirements: profile-onboarding

## Purpose

Give every learner a small account profile after OAuth sign-up and make it editable from Account settings. The first version captures the details Counterspell already owns (`display_name` and optional `country`), marks onboarding complete explicitly, and shows the approved skull as a placeholder avatar until durable object storage exists.

This is a **Requirements-First** feature spec. This spec-generation run executes no task and changes no implementation file.

## Sources and fixed decisions

- Brighton request, 22 Nov 2026: add a profile, capture user details, use the skull as the placeholder avatar, keep the avatar uneditable until S3/object storage exists, and provide an edit-profile flow plus OAuth onboarding.
- `.kiro/specs/auth/`: sign-in state, capabilities, OAuth callback security, CSRF and no-secret rules remain authoritative.
- `migrations/20261113110000_auth_core.sql`: `users.display_name` and `users.country` already exist.
- `web/static/art/creatures/skull-green-eyes.webp`: the existing approved nav skull and placeholder avatar.

## Scope

### Must

- Persist whether onboarding was completed without conflating it with sign-in.
- Let a signed-in, non-demo user edit `display_name` and `country` through the auth API.
- Return a fixed skull avatar URL and an `avatar_editable: false` flag from `/api/me`.
- Redirect a newly created OAuth account to `/onboarding` before the requested return destination.
- Render a profile section and edit-profile flow on Account settings without changing its section order after Sign-in methods.
- Keep validation, CSRF protection, capabilities and account isolation server-owned.

### Out of scope

- Avatar upload, crop, delete, Cloudflare/S3 wiring or provider avatar import.
- Public profiles, handles, bio/social links or a profile page for other users.
- Email/password sign-up or a separate account-registration flow.
- Layout/theme redesign beyond adding the profile controls in the existing Steel Night style.

## Glossary

- **Profile:** the signed-in learner's editable public-facing account fields: display name and optional country.
- **Profile completion:** an explicit `profile_completed_at` timestamp, set only by a successful profile update.
- **Placeholder avatar:** the static skull image returned by `/api/me`; it is not user-uploaded content.
- **Onboarding:** the signed-in `/onboarding` page shown once to a newly created OAuth account before its original return destination.

## Requirements

### PO-1: profile model and API

**User story:** As a learner, I want my basic details saved on my account, so Counterspell can show a real identity instead of a generated name only.

1. `users` SHALL persist a nullable `profile_completed_at` timestamp.
2. Existing users SHALL be marked complete by migration; new OAuth accounts SHALL start incomplete.
3. `GET /api/me` SHALL return `country`, `profile_complete`, `avatar_url` and `avatar_editable` in addition to the current user fields.
4. `PATCH /api/me/profile` SHALL accept a JSON body with `display_name` and optional `country`, update only the caller's row, set `profile_completed_at`, and return the updated user view.
5. `PATCH /api/me/profile` SHALL validate `display_name` after trimming to 1–80 characters and `country` to `null` or an uppercase ISO-style two-letter code.
6. `PATCH /api/me/profile` SHALL require authentication and the ordinary non-demo profile-edit capability; it SHALL return `demo_account_restricted` for the demo account.
7. Avatar upload or mutation input SHALL be absent from the API until object storage exists; `avatar_editable` SHALL be `false`.

### PO-2: OAuth onboarding

**User story:** As a new learner who signs up with GitHub or Google, I want to confirm my profile once, so Counterspell knows what to call me.

1. WHEN OAuth sign-in creates a new account, THE callback SHALL create the session and redirect to `/onboarding?return_to=<sanitized destination>`.
2. WHEN OAuth sign-in uses an existing account, THE callback SHALL preserve the existing `return_to` behavior unless the account is still incomplete.
3. `/onboarding` SHALL require a signed-in non-demo user; signed-out users SHALL be sent to `/login` with a safe return destination.
4. A successful onboarding profile save SHALL mark the profile complete and navigate to the sanitized return destination.
5. Onboarding SHALL use the same `PATCH /api/me/profile` seam as Account settings; it SHALL NOT write to `users` from the browser or another route.

### PO-3: account settings profile

**User story:** As a learner, I want to see and edit my profile from Account settings, so account details are in the expected place.

1. The Account identity area SHALL show the static skull avatar, display name, handle and email exactly once.
2. The Account page SHALL include an **Edit profile** action that reveals a keyboard-accessible form for display name and country.
3. The form SHALL preserve the existing sections and order, inserting the profile controls only inside the Account/identity region before Sign-in methods.
4. A successful save SHALL update the shared session store and show a status message without reloading the page.
5. The avatar SHALL be visible but non-interactive and SHALL include copy explaining that avatar upload is unavailable until storage is configured.
6. The demo account SHALL see the placeholder avatar and profile values but SHALL NOT be offered the edit action when `edit_profile` is false.

### PO-4: safety, accessibility and tests

**User story:** As a learner, I want profile editing to be safe and predictable, so my account cannot be changed by another user or malformed input.

1. CSRF protection SHALL apply to `PATCH /api/me/profile` exactly like other auth mutations.
2. A user SHALL never update another user's profile; tests SHALL include a canary second user.
3. The form SHALL use semantic labels, 44 px controls, visible status/error text and no colour-only result state.
4. TDD SHALL be used: failing backend validation tests and frontend form/component tests precede implementation.
5. `cargo test` for touched crates, `npm run check`, focused Vitest, lint/format and production build SHALL pass before merge.
6. The deployed app SHALL be checked on `https://counterspel.online` after merge.

## Acceptance evidence

- Migration adds `profile_completed_at` and preserves existing auth privileges.
- Backend tests cover validation, demo restriction, no-store response and user isolation where the DB harness allows.
- Frontend tests cover `/api/me` fields, the shared profile form, account edit flow and onboarding redirect behavior.
- No avatar upload control or storage credential exists.
- The same profile module serves onboarding and Account settings.
