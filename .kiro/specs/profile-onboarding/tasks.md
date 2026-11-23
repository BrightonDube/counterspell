# Tasks: profile-onboarding

All tasks below are implementation tasks for `requirements.md` and `design.md`. No task is executed by this spec-generation run. `[S]` means sequence-sensitive; `[P]` means parallel-safe after dependencies. Keep the one-PR-at-a-time workflow: one focused PR per task or tightly bound task pair, merge, clean the branch, then verify live.

## Rules for every task

- Preserve the canonical workspace layout and auth security boundaries.
- Follow `.kiro/steering/`, `docs/`, and the TDD skill: write a failing test first for every behavior change.
- Never let the browser choose the user id; handlers use the authenticated extractor only.
- Avatar remains static and uneditable until real object storage is configured.
- Run focused tests, then `cargo test` for touched crates, `npm run check`, focused Vitest, lint/format and build before the PR.

## Wave 0: profile backend

- [ ] **T01 [S] (35 min) Add profile persistence and validation**
  - Files: `migrations/20261204120000_profile_onboarding.sql`, `crates/server/src/auth/{capabilities.rs,profile.rs,mod.rs,errors.rs}`, auth tests.
  - Add `profile_completed_at`, backfill existing users, add `edit_profile` capability, and implement the validated `ProfilePatch` module. Reuse `invalid_json` for malformed profile input rather than adding a new public error.
  - Verify: failing migration/schema check and validation tests first; display-name trim/limits and country casing/limits behave exactly; demo lacks `edit_profile`.
  - Requirements: PO-1.1, PO-1.5, PO-1.6.
  - PR alone: yes.

- [ ] **T02 [S] (30 min) Expose and update `/api/me` profile**
  - Files: `crates/server/src/auth/handlers/{me.rs,profile.rs,mod.rs}`, `crates/server/src/auth/mod.rs`, `web/src/lib/auth/client.js`, HTTP integration tests.
  - Return `country`, `profile_complete`, `avatar_url` and `avatar_editable`; add `PATCH /api/me/profile` behind auth/CSRF that updates only the caller and returns `MeUser`.
  - Verify: failing handler tests first; no-store remains, hidden fields are absent, demo gets `demo_account_restricted`, a canary second user is unchanged.
  - Requirements: PO-1.3–PO-1.7, PO-4.1, PO-4.2.
  - PR alone: yes.

- [ ] **T03 [S] (25 min) Redirect new OAuth accounts to onboarding**
  - Files: `crates/server/src/auth/handlers/oauth.rs`, OAuth tests.
  - Distinguish created from existing accounts and send a newly created account to `/onboarding?return_to=<encoded destination>` after session creation, preserving 2FA behavior.
  - Verify: failing OAuth seam test first; existing users keep `return_to`, denied/failed flows unchanged.
  - Requirements: PO-2.1–PO-2.2.
  - PR alone: yes.

## Wave 1: profile UI

- [ ] **T04 [S] (35 min) Add the shared profile form**
  - Files: `web/src/lib/auth/{client.js,ProfileForm.svelte,profile.test.*}`, `web/src/routes/settings/account/+page.svelte`, account page tests.
  - Build a labelled display-name/country form that calls `PATCH /api/me/profile`; use it in the Account identity disclosure, show the fixed skull avatar and unavailable-upload copy, update `session.user` on save, and hide edit for demo accounts.
  - Verify: failing component tests first; accessible labels, 44 px submit, avatar copy, success/problem roles and demo behavior pass; existing account sections stay ordered.
  - Requirements: PO-1.3, PO-3, PO-4.3.
  - PR alone: yes.

- [ ] **T05 [S] (30 min) Add the OAuth onboarding page**
  - Files: `web/src/routes/onboarding/+page.svelte`, onboarding tests, `web/src/lib/auth/return-to.js` only if needed.
  - Render the standalone onboarding surface, require a session, use `ProfileForm`, preserve and sanitize `return_to`, and navigate to `/dashboard` when no destination exists.
  - Verify: failing route tests first for signed-out redirect, successful save destination, malformed return destination, and demo fallback; `npm run check` and focused Vitest.
  - Requirements: PO-2.3–PO-2.5, PO-4.3.
  - PR alone: yes.

- [ ] **T06 [S] (20 min) Verify profile-onboarding and tick the spec**
  - Files: only fixes and this spec's checklist.
  - Re-read touched modules with the codebase-design vocabulary; keep the shared seam only because onboarding and settings both call it. Verify desktop/mobile, keyboard paths, live deployment and no stray worktrees.
  - Verify: focused Rust tests, full web suite, `npm run check`, lint/format, build, CI and Devin Review are green; all PO requirements are checked below.
  - Requirements: all PO requirements.
  - PR alone: only if cleanup/evidence changes are needed.

## Completion checklist

- [ ] `profile_completed_at` is persisted, backfilled and set only by the profile patch path.
- [ ] `/api/me` exposes profile fields and the fixed, uneditable skull avatar.
- [ ] `PATCH /api/me/profile` is authenticated, CSRF-guarded, validated, user-isolated and demo-restricted.
- [ ] New OAuth accounts land on onboarding before the requested destination.
- [ ] Account settings and onboarding share the same profile form and API seam.
- [ ] Avatar upload is absent but explicitly marked unavailable.
- [ ] Focused backend tests, web tests, checks, build, CI, review and live verification pass.
