# Design: profile-onboarding

## Design thesis

The profile is a small deep module: the auth handler owns validation, persistence and completion; the browser owns only the shared form and destination navigation. Avatar upload is represented as an explicit capability gap, not as dead UI pretending storage exists.

## Data model

Add `users.profile_completed_at timestamptz`. Existing rows are marked complete in the migration so current OAuth users are not forced through a surprise onboarding screen; newly created OAuth rows start `NULL`. The existing `display_name` and `country` fields are reused instead of adding a separate profile table.

`/api/me` extends `user` with:

```json
{
  "country": "ZA",
  "profile_complete": true,
  "avatar_url": "/art/creatures/skull-green-eyes.webp",
  "avatar_editable": false
}
```

`Capabilities` gains `edit_profile`, false for demo and true for normal users.

## Profile seam

Add `crates/server/src/auth/profile.rs` with two small interfaces:

- `ProfilePatch::try_new(display_name, country)` validates and normalises trim/casing.
- `update_profile(pool, user, patch, now)` updates only `display_name`, `country`, `profile_completed_at` and `updated_at`.

`PATCH /api/me/profile` accepts `{ display_name, country }`, calls `Capabilities::require(EditProfile)`, updates the authenticated `AuthUser`, and returns the same `MeUser` shape as `GET /api/me`. The handler remains the only write path, so settings and onboarding cannot drift.

Country is `NULL` or exactly two ASCII uppercase letters. This matches the existing `^[A-Z]{2}$` constraint without pretending to ship a country catalogue.

## OAuth redirect seam

`sign_in` needs to know whether the decision created a user. Change `create_account` to return `(UserId, bool)` or a small `CreatedUser`/`ExistingUser` outcome. For a created account, after `complete_primary` succeeds and no 2FA step is pending, the callback redirects to:

```text
/onboarding?return_to=<urlencoded sanitized destination>
```

Existing users keep `flow.return_to()`. No extra cookie is needed: the normal session cookie authenticates the onboarding page.

## Shared frontend form

Add `web/src/lib/auth/ProfileForm.svelte` because there are two concrete callers:

- `/onboarding`: required first-save flow, then navigates to `return_to`.
- `/settings/account`: optional disclosure under the identity header, updates `session.user` in place.

The form accepts `initialDisplayName`, `initialCountry`, `submitLabel`, and an `onSaved` callback. It calls `patchProfile` in `client.js`; it does not fetch `/api/me` itself. Labels stay visible, inputs use existing field styles, and status/problem text is announced with roles.

`/api/me` consumers are tolerant: `profile_complete` may be absent on an old server build, so the client only uses it to show incomplete state rather than inventing completion.

## Account settings layout

The existing header stays first. Inside the identity region:

- left: skull avatar (`/art/creatures/skull-green-eyes.webp`, fixed 96×96, `alt=""`) beside the current name/handle/email block;
- below: display name, country, avatar availability copy, and an **Edit profile** disclosure;
- no reordering of `Sign-in methods`, `Security`, `Sessions`, or `Delete account`.

The edit button is hidden for demo accounts. A successful patch replaces `session.user` with the response and shows “Profile saved.”

## Onboarding page

Add `web/src/routes/onboarding/+page.svelte` under the standalone auth visual shell style. On mount it loads `session`; signed-out users go to `/login?return_to=%2Fonboarding%3Freturn_to%3D...`; demo users go to `/`. The page explains the fixed skull avatar, renders `ProfileForm`, then navigates to `safeReturnTo(return_to)` or `/dashboard`.

## Migration and privileges

New migration `20261204120000_profile_onboarding.sql`:

- `ALTER TABLE users ADD COLUMN IF NOT EXISTS profile_completed_at timestamptz`;
- mark existing rows complete;
- update the `counterspell_server` update grant to include `profile_completed_at` if the privilege test requires explicit column grants.

## Safety and failure modes

- `PATCH` remains behind the JSON body guard and auth cookie middleware.
- Unique collisions are impossible for the editable fields; handle is not editable in this slice.
- Empty display name or malformed country returns `invalid_json` with a safe message rather than a database error.
- `avatar_url` is static server output, never request input.
- A failed onboarding save keeps the learner on `/onboarding` with a normal error.

## Test strategy

TDD order:

1. Unit tests for `ProfilePatch` validation and normalisation.
2. Auth capability test for `edit_profile`.
3. DB-backed route test, where available: patch only caller, set completion, demo rejection and no-store.
4. Web tests for `ProfileForm`, Account settings fields/action, and onboarding redirect/save.
5. OAuth handler test or focused seam test proving a newly created user redirects to onboarding.

## Risks

- Backfilling all existing users avoids breaking production, but genuinely incomplete legacy profiles will not be forced to onboard. This is the deliberate migration choice.
- `country` is a code, not a full country selector. The API stays forward-compatible if a catalogue is added later.
- Avatar storage is future work; the response makes its immutability explicit rather than shipping a broken upload control.
