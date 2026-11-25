# Tasks: chess-profile-polish

All tasks are pending. Requirements-first spec for #131. [S] sequence-sensitive; [P] parallel-safe after baseline. Follow AGENTS.md, docs, scoped steering, Impeccable/Svelte/TDD and one-PR-at-a-time workflow. No application implementation in the spec PR.

## Wave 0 — baseline
- [ ] **CPX01 [S] Capture current chess and profile evidence.** Record current sprite, actual board colors/geometry, bot/puzzle behavior and profile mobile/desktop screenshots. Confirm current tests, existing profile schema/API and repository constraints. Requirements: CPX-1–4. Verify: baseline evidence recorded; no private data.

## Wave 1 — independent vertical slices
- [ ] **CPX02 [P] Redesign and verify shared chess sprite.** Depends CPX01. Add failing sprite/geometry regressions; produce all twelve original pieces plus crown in existing contract; render both sides on both colors, both routes and target viewport sizes; verify settled bounds, move/drag, promotion, orientation, bot/puzzle, markers, reduced motion and accessibility. Request owner visual review. Requirements CPX-1, CPX-4. PR alone: yes.
- [ ] **CPX03 [P] Redesign responsive identity and remove internal messaging.** Depends CPX01. Add failing account/onboarding rendering tests; centered mobile avatar, name/handle hierarchy, readable country, private email, static avatar and no storage message; preserve security section order and capability gates. Verify narrow/desktop, long text, keyboard and screenshots. Requirements CPX-2, CPX-4. PR alone: yes.
- [ ] **CPX04 [P] Improve shared profile edit experience end to end.** Depends CPX01; coordinate with CPX03 if touching the same files, merge sequentially. Add failing tests; Full name label, searchable accessible country picker, clear/cancel, saving/error feedback and saved session update. Preserve API/schema and legacy codes; test authenticated UI persistence/reload, onboarding, demo restrictions and canary isolation. Requirements CPX-3, CPX-4. PR alone: yes.

## Wave 2 — closeout
- [ ] **CPX05 [S] Verify integration and production.** Depends CPX02–04. Run all repository gates, review evidence and approved visual design, deploy via normal pipeline, verify live counterspel.online bot/puzzle and signed-in profile via UI with permitted account. Record exact commit, checks, screenshots and limitations. Do not tick unverified criteria or close #131 until complete.

## Completion checklist
- [ ] Original chess pieces legible, scalable and contained at all target sizes.
- [ ] Both chess modes and existing game behavior preserved.
- [ ] Mobile identity centered; name first; username separate; developer copy absent.
- [ ] Full name and country edit/save/cancel work and persist; auth/onboarding unchanged.
- [ ] Regression tests, CI, code review, owner visual approval and production verification recorded.
