Create a Requirements-First feature spec named ide-workspace. This is the central Counterspell experience and the main Costume Contest showcase. UI will be implemented partly outside Kiro, so keep tasks granular, label UI-only tasks "[UI]", and mark independent UI tasks so they can run in parallel.
Context: docs/product-brief.md core loop, docs/requirements.md IDE-1–IDE-6, GHO-1–GHO-6, FAM-1, FAM-3–FAM-6, VAL-5, docs/system-design.md §5, §8, §8b, §9 and §16, and docs/palette.md (Monaco "Crypt" theme). Use SvelteKit, Svelte 5, Monaco, Bits UI, Tailwind 4 and GSAP. Follow Steel Night. No React, Next.js or Pyodide.
User journey:
1. Open a problem with a clear statement, constraints and starter function (Python or JavaScript at launch; Java/C++ appear only if the stretch drivers ship).
2. Write and edit without losing drafts (per problem and language).
3. Run sample tests and inspect input / expected / got.
4. Submit and receive real execution status updates over SSE (replay with Last-Event-ID, dedupe by seq).
5. On a wrong answer, animate the transition into Ghost Hunting.
6. Show the visible shrink animation: replay the recorded shrink steps from the first failing input down to the minimized counterexample (only real recorded steps; static list under reduced motion).
7. Show the Bug Autopsy card: failing input, expected vs actual with the differences highlighted, and the conceptual mistake (authored explanation, or an AI explanation labelled as AI).
8. Offer escalating authored hints and a button to rerun the ghost as a custom test.
9. Let the learner correct the code and resubmit.
10. Show a successful resolution, the familiar's measured mood and its labelled AI speech bubble, and update progress.
Design requirements: a cinematic but functional ghost reveal; never animate or display fabricated execution results (design fixtures are allowed only in clearly labelled prototype routes); calm, fast, keyboard-accessible editor interactions; reduced-motion equivalents; status labels that don't rely on colour alone; output rendered as plain text only; the familiar's mood comes only from measured run events (no live typing guess in this spec); lazy-load Monaco and heavy animation assets; usable on a mid-range laptop and a tablet.
Deliverables: interaction flow, responsive layout, component boundaries, animation state machine, loading/error/empty states, keyboard behaviour and accessibility acceptance tests.
Build and test against the actual Axum API, not mocked success responses (the early [UI] shell may use labelled fixtures until the API exists).
Definition of done: a judge can submit a wrong solution, watch the real shrink, read a real minimized counterexample in the Bug Autopsy, fix the code and reach Accepted without a page reload or manual intervention.

This run is non-interactive (Kiro CLI). Read the referenced docs and .kiro/steering/ yourself. Write the complete spec (requirements.md, design.md, tasks.md) under .kiro/specs/ide-workspace/ in one go, then stop. This is the system-design stage: do NOT implement any task, and do not create or change any file outside .kiro/specs/ide-workspace/. Keep consistent with the existing .kiro/specs/repo-skeleton/ (layout, crate names). Put open questions for Brighton at the end of design.md under "Open questions", each with the choice you assumed.
