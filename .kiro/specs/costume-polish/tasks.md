# Tasks: costume-polish

All tasks below are implementation tasks for `requirements.md` and `design.md`. No task is executed by this spec-generation run. `[S]` means sequence-sensitive; `[P]` means parallel-safe after dependencies. Keep the one-PR-at-a-time workflow: one focused PR per task or tightly bound task pair, merge, clean the branch, then verify live.

## Rules for every task

- Preserve the canonical six-package workspace and existing component/layout order.
- Follow `docs/palette.md`, `.kiro/steering/frontend.md`, `.kiro/steering/security.md`, and the TDD skill: write a failing test first for every behavior change.
- Use semantic palette tokens only; no raw hex in components.
- Use only existing `web/static/` assets; decorative images get `alt=""`, explicit dimensions and appropriate lazy loading.
- Keep output as plain text and hidden-test fields absent. Do not expose hidden input, expected output, stdout or stderr.
- Motion is short, purposeful and non-looping in Operate surfaces; every animation has a `prefers-reduced-motion` path.
- Run focused tests first, then `npm run check`, `npm run lint` where available, and the relevant backend suites before the PR.

## Wave 0: costume surfaces

- [x] **T01 [S] (30 min) Add the nav creature mark**
  - Files: `web/src/lib/shell/AppShell.svelte`, a small component test, and CSS if needed.
  - Render `art/creatures/skull-green-eyes.webp` beside the Counterspell wordmark as a compact decorative mark, preserving all nav links and keyboard order. Add a bounded hover/focus-within eye glow or flicker, static under reduced motion. Hide it only when it would crowd narrow navigation.
  - Verify: failing component test asserts decorative alt, source, dimensions and nav content; then `npm run check` and focused Vitest.
  - Requirements: CP-1, CP-3.
  - PR alone: yes.

- [x] **T02 [S] (30 min) Animate settings without changing layout**
  - Files: `web/src/routes/settings/account/+page.svelte`, account/settings CSS, component tests.
  - Animate the existing crypt key on hover/focus (short turn + glow), give provider-link activation a bounded key/unlock affordance, and add at most small existing object art where it does not reorder content. Keep every section and control in place.
  - Verify: tests assert classes/ARIA are unchanged and output order is preserved; manual visual check at desktop and 375 px.
  - Requirements: CP-1, CP-2, CP-5 accessibility.
  - PR alone: yes.

## Wave 1: dashboard calendar

- [x] **T03 [S] (30 min) Return real dashboard activity days**
  - Files: `crates/server/src/dashboard/{types.rs,query.rs,handler.rs}` and dashboard database/API tests.
  - Add a bounded `activity_days` list of UTC `YYYY-MM-DD` values derived from the authenticated user's completed Run/Submit rows. Reuse the existing dashboard query/CTEs where practical and reject another user's dates with canary tests.
  - Verify: write failing query/API tests first; duplicate days collapse, unfinished/internal-error rows do not count, user isolation holds, response stays stable.
  - Requirements: CP-4.1, CP-4.2.
  - PR alone: yes.

- [x] **T04 [S] (30 min) Build the dashboard practice calendar**
  - Files: `web/src/lib/dashboard/{types.ts,PracticeCalendar.svelte,calendar.ts,tests}` and `DashboardScreen.svelte`.
  - Parse and validate `activity_days`, build a recent UTC month/heatmap view, mark today and active days, and expose the same facts in text. Preserve all existing dashboard sections and only insert the calendar where it fits the current flow.
  - Verify: failing pure tests for month bucketing/UTC boundaries and component tests for empty, active, keyboard-readable and malformed-data states; `npm run check`, focused Vitest.
  - Requirements: CP-4.3–CP-4.6.
  - PR alone: yes.

## Wave 2: workspace console and load health

- [x] **T05 [S] (30 min) Extend public sample console detail safely**
  - Files: `crates/common/src/events.rs`, `crates/judge/src/sandbox/{records.rs,sandbox_executor.rs}`, `crates/judge/src/queue/finish.rs`, server read-model mapping and canary tests.
  - Extend the trusted public sample detail only if needed so bounded stderr can reach the browser for sample tests. Keep hidden tests unable to carry input/expected/stdout/stderr. If stdout is not separately represented, keep the existing `got` semantics and do not fabricate a channel.
  - Verify: failing common/judge/server contract tests first; hidden canary rows still have no output fields; caps are preserved.
  - Requirements: CP-5.2, CP-5.3.
  - PR alone: yes.

- [x] **T06 [S] (30 min) Add the collapsible workspace console**
  - Files: `web/src/lib/runs/types.ts`, `web/src/lib/workspace/{console-view.ts,ConsolePanel.svelte,WorkspaceShell.svelte}`, tests and CSS.
  - Parse the public console fields, build ordered bounded console entries, and render a keyboard-accessible expandable/collapsible read-only console associated with the editor. Auto-open once when a run produces content, never after an explicit collapse for that run. Keep Run/Submit/results/ghost/hints/Familiar unchanged.
  - Verify: failing parser/view/component tests first, including hostile text staying plain and hidden rows carrying no output; `npm run check` and focused Vitest.
  - Requirements: CP-5.1–CP-5.6.
  - PR alone: yes.

- [ ] **T07 [P] (30 min) Verify and improve first workspace load**
  - Files: `web/src/lib/editor/{MonacoEditor.svelte,monaco-loader.ts}`, `web/src/lib/workspace/*`, bundle/performance test support only where a verified blocker exists.
  - Inspect lazy Monaco loading, current chunk output and timing marks. Fix one confirmed first-load blocker if present (for example duplicate Monaco/theme loading or unnecessary eager imports). Do not guess a rewrite.
  - Verify: record bundle/measurement evidence; shell/editor/first-event/terminal marks still exist; workspace and performance tests pass.
  - Requirements: CP-6.
  - PR alone: yes.

## Wave 3: freeze-safe polish checkpoint

- [ ] **T08 [S] (30 min) Costume-polish verification and code organisation**
  - Files: only fixes and this spec's checklist.
  - Re-read the touched modules with the codebase-design vocabulary; extract a shared component/helper only when two or more concrete callers justify it. Verify desktop/mobile visual checks, keyboard paths, reduced motion, live deployment and no stray worktrees.
  - Verify: full focused web suite, touched Rust tests, build, CI and Devin Review are green; all CP requirements are checked below.
  - Requirements: all CP requirements.
  - PR alone: only if cleanup/evidence changes are needed.

## Completion checklist

- [ ] Asset use is counted, existing, bounded and non-jack-o'-lantern for nav.
- [ ] Settings key and provider interaction are animated without layout changes.
- [ ] Dashboard renders a real activity calendar with text equivalent.
- [ ] Workspace console is expandable/collapsible and public-output safe.
- [ ] Run/Submit, tests, ghost, autopsy, hints and measured Familiar are unchanged.
- [ ] Reduced motion, keyboard access, 200% zoom and 44 px targets pass.
- [ ] Each merged task was verified on `https://counterspel.online`.
