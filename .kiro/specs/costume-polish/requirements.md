# Requirements: costume-polish

## Purpose

Make the signed-in Counterspell UI feel alive for the competition's costume category without changing the approved Steel Night world or the current page layouts. The work adds existing character art and purposeful feedback motion, restores the dashboard activity calendar, and completes the promised IDE console while preserving the measured ghost, submit and Familiar flows.

This is a **Requirements-First** feature spec. The files in this directory define the feature; this spec-generation run executes no task and changes no implementation file.

## Sources and fixed decisions

- Brighton request, 22 Nov 2026: keep the existing layout, animate it, add more graphics from the asset folder, add a scary nav graphic other than the jack-o'-lantern, restore the NeetCode-inspired dashboard calendar, and make the IDE behave more like a real playground with an expandable/collapsible console.
- `docs/product-brief.md`: horror-themed practice product, calm IDE, cinematic landing, competition costume direction.
- `docs/palette.md`: semantic Steel Night tokens; moss/candle accents; bright green only for success; red only for errors; calm dashboard/IDE motion.
- `docs/system-design.md` and `docs/requirements.md`: IDE includes a console; stdout/stderr/test output is plain text and bounded; hidden test output must never reach the browser.
- `.kiro/specs/ide-workspace/`: lazy Monaco, existing Run/Submit lifecycle, result panel, ghost, autopsy, hints and Familiar behavior are protected seams.
- `.kiro/specs/dashboard-fsrs/`: dashboard data comes from the authenticated learner's own completed submissions and must not fabricate activity.
- `.kiro/specs/auth/` and `web/src/routes/settings/account/+page.svelte`: settings page is an authenticated Operate surface.
- `web/static/art/`: existing approved art. Inventory count for this spec: **186 image files**, **62 logical assets** after grouping format/size variants.

## Scope

### Must

- Existing art only; no new generated or third-party imagery.
- Settings/account graphics and interaction feedback without changing the information architecture or control order.
- A non-jack-o'-lantern nav creature graphic.
- A dashboard practice calendar backed by real activity days.
- A collapsible/expandable workspace console that displays safe public run output.
- Reduced-motion and keyboard-accessible alternatives for every animation and disclosure.
- First workspace interaction must not regress; keep Monaco lazy-loaded and identify/remove an obvious first-load blocker only when verified.

### Out of scope

- Changing the approved Steel Night palette, fonts, routes, or component order.
- A new calendar endpoint outside `/api/dashboard`.
- Client-side execution, a terminal emulator, browser code execution, or arbitrary interactive stdin.
- Hidden-test input, expected output, stdout, stderr, or private judge diagnostics.
- Neon/purple themes, cartoon mascots, or ambient editor motion.

## Glossary

- **Logical asset:** one artwork after grouping SVG/PNG/WebP/AVIF and size variants.
- **Activity day:** a UTC calendar date with at least one completed Run or Submit owned by the signed-in learner.
- **Practice calendar:** the dashboard's month/heatmap view of activity days, inspired by NeetCode's streak calendar.
- **Console:** the workspace's read-only output region for bounded public compile output and public sample-test stderr/stdout detail.
- **Public output:** output the run lifecycle is already allowed to show for a sample test or a compile result. Hidden-test output remains forbidden.
- **Costume motion:** short, purposeful transform/opacity/filter feedback in the existing Steel Night language.

## Requirements

### CP-1: asset-backed costume polish

**User story:** As a learner, I want the signed-in app to feel dressed for the horror competition, so the product reads as an authored haunted workspace rather than a plain dashboard.

1. THE SYSTEM SHALL use only files already under `web/static/` or `web/src/assets/` for new graphics.
2. THE SYSTEM SHALL prefer WebP/AVIF stills or the existing SVG source when a component needs a small asset; it SHALL NOT ship multiple format variants of the same logical asset to one component.
3. Every decorative image SHALL have an explicit size, `alt=""`, and a loading strategy that does not delay first interaction above the fold.
4. THE SYSTEM SHALL keep semantic palette tokens and SHALL NOT introduce purple, raw hex colors in components, or a second display font for product UI.
5. THE SYSTEM SHALL add at least one non-jack-o'-lantern creature graphic to signed-in navigation; the approved candidate is `art/creatures/skull-green-eyes.webp`, with `raven.webp` as the fallback.
6. THE SYSTEM SHALL preserve all existing text, controls, landmarks, tab order and page layout except for the new calendar and console regions required by CP-4 and CP-5.

### CP-2: settings/account costume feedback

**User story:** As a learner managing sign-in methods, I want subtle haunted feedback, so the settings page feels polished without slowing the task.

1. WHEN a learner points at or keyboard-focuses the account crypt key, THE SYSTEM SHALL animate the key with a short spin/glow affordance.
2. WHEN a learner activates an available provider link, THE SYSTEM SHALL provide a bounded visual transition that communicates the key connecting the account before navigation or while the provider action is pending.
3. Section entrances and disclosure controls MAY animate only to explain state; they SHALL NOT add a long page-load choreography or delay account actions.
4. Every settings animation SHALL finish within 500 ms, use transform/opacity/filter where practical, and have a reduced-motion alternative that preserves the state change.
5. The settings page SHALL keep its existing identity, methods, security, passkeys, sessions and delete sections in the same order.

### CP-3: signed-in navigation graphic

**User story:** As a learner, I want a small scary character in the app chrome, so the signed-in app keeps the costume world outside the landing page.

1. THE navigation SHALL render one compact creature mark beside the Counterspell wordmark on signed-in app screens.
2. The creature mark SHALL be decorative (`alt=""`), keyboard-inert, and SHALL NOT reduce the size or clarity of navigation links.
3. The creature mark MAY have a calm hover/focus-within glow or eye flicker, but SHALL NOT animate continuously while the learner works.
4. The creature mark SHALL keep contrast and spacing on 320 px, tablet and desktop widths and SHALL collapse or hide only if it would crowd navigation.

### CP-4: dashboard activity calendar

**User story:** As a learner, I want to see which days I practised, so streak progress is visible at a glance.

1. `GET /api/dashboard` SHALL return a bounded, UTC-normalised list of real activity days owned by the authenticated learner.
2. Activity days SHALL be derived from the same completed-run source as the existing streak and SHALL NOT include another user's activity or unfinished/infrastructure-failed runs.
3. The dashboard SHALL render a calendar-style heatmap for the recent practice window, marking today, activity days, and the current streak context in words as well as colour.
4. WHEN the learner has no activity, THE calendar SHALL render an empty but understandable state and SHALL NOT fabricate practice days.
5. The calendar SHALL be keyboard-readable or expose an equivalent text list of active days.
6. The calendar SHALL keep the dashboard's calm 120–500 ms motion rules and SHALL NOT use ambient looping motion.

### CP-5: workspace console

**User story:** As a learner, I want a playground-style console, so I can see bounded program output while keeping the editor large.

1. THE workspace SHALL add a console region associated with the editor, expandable and collapsible through a visible button with `aria-expanded` and keyboard support.
2. The console SHALL show compile output when a run reports it and bounded stderr/stdout detail only for public sample tests, with hidden tests represented by no output fields.
3. Program output SHALL render as plain text, use the existing display caps, and never become executable HTML.
4. The console SHALL default to collapsed or a compact state that preserves a large editor area, SHALL remember expansion only for the current workspace mount, and SHALL auto-expand only when a run produces console content.
5. The console SHALL preserve Run samples, Submit, ghost rerun, tests, ghost/shrink trace, Bug Autopsy, hints, and the measured Familiar. It SHALL NOT replace the results panel or add ambient editor motion.
6. The console SHALL expose an accessible name, text status when empty, and a clear way to distinguish stdout/stderr/compile sections.

### CP-6: first-load performance

**User story:** As a learner on a normal connection, I want the workspace to become usable quickly, so visual polish does not make the IDE feel heavier.

1. Monaco and heavy familiar/motion assets SHALL remain lazy-loaded; the initial application shell SHALL NOT eagerly fetch them.
2. Any new art SHALL be bounded and lazy-loaded unless it is the tiny nav creature mark.
3. The workspace SHALL keep its existing timing marks and add or preserve a mark that can compare editor readiness before and after this spec.
4. The implementation SHALL verify bundle output and route behavior; a first-load blocker may be fixed only after it is identified by code path, bundle analysis, or performance marks.
5. The implemented UI SHALL pass the existing workspace checks and SHALL NOT regress 200% zoom, keyboard navigation, reduced motion, or the 44 px target requirement.

## Acceptance evidence

- Unit/component tests cover each new view model and safety boundary before implementation, following the repo's TDD rule.
- Backend contract tests cover activity days and any new public console payload field.
- `npm run check`, focused Vitest suites, `cargo test` for touched crates, formatting and lint pass.
- Browser verification at desktop and mobile widths confirms preserved layout, keyboard and reduced-motion behavior.
- The live `https://counterspel.online` deployment is checked after each merged task.
