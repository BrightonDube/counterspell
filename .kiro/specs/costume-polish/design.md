# Design: costume-polish

## Design thesis

The costume is **haunted workbench**, not haunted theme park. Existing surfaces stay in place. The new motion explains one action at a time: the key responds because it unlocks sign-in, the skull marks the signed-in crypt, the calendar exposes practice history, and the console opens only when the learner needs output.

## Asset inventory

Current static inventory: **186 image files** under `web/static/`, **62 logical assets** after grouping format and size variants. Useful groups:

- `art/creatures/`: skull-green-eyes, skull-blood-eyes, wraith variants, raven, grave-hand, spider, cat, reaper.
- `art/objects/`: crypt-key, grimoire, cauldron, candles, lantern, cursed CRT, tombstone, coffin, chain, potion, brain jar, crystal ball, eyeball, skeleton keyboard, bug, hourglass.
- `art/fx/`: fog, smoke, embers, bats, crows, lightning, spores.
- `art/familiar/` and `art/landing/`: keep for their existing roles.

Selections:

| Surface | Asset | Reason |
| --- | --- | --- |
| Nav | `art/creatures/skull-green-eyes.webp` (fallback `raven.webp`) | Scary, compact, not the overused jack-o'-lantern |
| Settings identity | existing `art/objects/crypt-key-128.webp` | Already present; animate rather than replace |
| Settings sections | small object art such as `brain-jar-128.webp` or `crystal-ball-128.webp` | Adds costume without changing text order |
| Dashboard calendar | no creature image needed | Data density stays primary |
| Workspace console | `skeleton-keyboard.webp` optional tiny affordance only if it does not crowd controls | Console is a tool, not a hero panel |

## Motion plan

- **Focal moment:** the crypt key in Account. Hover/focus gives a bounded turn and candle glow; activating a provider emits the same key-forward pulse so the control feels like it is unlocking the link.
- **Continuity:** disclosures and the workspace console use a 150–250 ms opacity/transform transition; no height animation if it causes layout thrash.
- **Feedback:** connected states pulse once; calendar cells and console disclosure do not animate continuously.
- **Budget:** no ambient loops in Operate surfaces. Transform/opacity/filter only. Reduced motion keeps a static key and instant disclosure state.

## Module and component design

### Shared art affordance

Add a small reusable component under `web/src/lib/art/` only if two surfaces need the same behavior. Otherwise keep each component local. Images use explicit dimensions, `alt=""`, semantic classes, and `loading="lazy"` below the fold. The nav mark is eager or preloaded only if measured cheap enough.

### Settings/account

`+page.svelte` keeps its current markup and ordering. The key image gets a wrapper class that applies a short rotate/glow on hover/focus-within. Provider-link buttons get a pending class while `busy === 'methods'`; any decorative key motion lives in CSS and does not alter navigation or focus. Optional section glyphs use existing object art and do not replace the section heading text.

### Navigation

`AppShell.svelte` gets a compact decorative skull beside the wordmark. The link content and nav order do not change. The skull can add an eye glow on `header:hover`/`focus-within`; reduced motion is static. If it crowds the 320 px nav, it hides rather than wraps the links.

### Dashboard calendar

Backend: extend the dashboard response with `activity_days` or equivalent bounded array of UTC `YYYY-MM-DD` dates, derived from the same completed submissions used by the streak. Keep it under one dashboard read if possible; do not add a second browser round trip.

Frontend: `DashboardScreen.svelte` receives a `PracticeCalendar.svelte` under `web/src/lib/dashboard/`. A pure calendar view-model turns the activity-day list and today's date into weeks/months. The UI shows a recent month/heatmap and an equivalent textual summary. Today is marked; active days use a moss/candle fill and a text key. The component handles empty activity and invalid/missing data without inventing dates.

### Workspace console

The console is a read-only region below the editor controls and above/near results without replacing `TestPanel`.

Data seam:

- `DonePayload.compile_output` is already public and bounded.
- Public sample-test stderr may be exposed only by extending the trusted public `SampleDetail` contract. The database already has `stderr_snip`, but public rows use `public_detail`; any contract change must keep hidden tests with no detail fields.
- If real stdout is not separately available, label the existing `got` as sample output and do not fake a second stdout channel. Never imply hidden output exists.

Components:

- `console-view.ts` converts run stream/view data into console lines: compile output first, then public sample diagnostics in test order, truncating to the existing display cap.
- `ConsolePanel.svelte` renders a bordered read-only region with a disclosure button, status text, and monospace plain-text blocks.
- `WorkspaceShell.svelte` owns `consoleOpen` state only for the current mount. It opens automatically when a terminal run leaves console content; it never reopens while a learner explicitly collapsed it for that same run.
- `EditorPanel.svelte` may receive the console as a snippet only if that keeps ownership simple; otherwise the shell owns it. Do not move Run/Submit or test/ghost/hint/familiar behavior.

### First-load performance

Keep `MonacoEditor.svelte` lazy-loaded and avoid importing animation libraries into the signed-in shell. Measure with existing marks and build chunks. Candidate cheap wins, only if verified:

- Avoid eagerly importing heavy components into routes that do not need them.
- Reuse a loaded Monaco worker/theme instead of re-defining it.
- Keep new art in WebP and below a few KiB where possible.
- Do not preload large creature assets on routes that do not show them.

## Accessibility and safety

- Every decorative image is `alt=""`; every control has an accessible name.
- Console disclosure uses `aria-expanded`/`aria-controls`; a text status announces empty output.
- Reduced motion preserves state feedback without spatial movement.
- Output remains escaped text nodes; no `{@html}` for program/compiler output.
- Hidden test rows still contain only ordinal, verdict and time. Any test that supplies forbidden keys to a hidden row must fail parsing.
- Color is never the only indicator: calendar has labels/legend and console sections have headings.

## Test strategy

TDD order:

1. Pure view-model/parser tests for calendar date bucketing, console rows and hidden-output rejection.
2. Backend contract/query tests for activity days and sample stderr propagation if the public contract is extended.
3. Component tests for nav asset markup, settings classes, calendar states and console disclosure.
4. Existing workspace regression tests: Run/Submit, stream replay, ghost, hints, Familiar and output safety.

## Risks

- Extending `SampleDetail` can accidentally widen the public surface. Mitigate with explicit hidden-row forbidden-key tests and canary assertions.
- A calendar can imply precision the API lacks. Return explicit UTC dates and render an empty state when absent.
- Extra motion can violate calm IDE rules. Keep motion bounded, purposeful and non-looping.
- Asset inventory includes generated build copies. Use only `web/static/`, never `web/build/`.
