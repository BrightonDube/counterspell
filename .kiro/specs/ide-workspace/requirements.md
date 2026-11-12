# Requirements: ide-workspace

## Introduction

The IDE workspace is Counterspell's central learner experience and the main Costume Contest showcase. It turns a published problem, a learner's draft, the trusted judge's persisted run events, and the ghost engine's real minimized counterexample into one calm, keyboard-accessible loop:

`problem → edit → Run samples → Submit → live judge → Ghost Hunting → shrink trace → Bug Autopsy → hint/custom run → resubmit → Accepted → measured Familiar`

This is a Requirements-First feature spec. It defines the browser workspace, its API contracts and integration seams, and the tests that prove the complete journey. It does not change the canonical six-crate layout.

The implementation must build on these existing decisions and specs:

- `repo-skeleton`: `web/` is the SvelteKit 5 static app served by `counterspell-server`.
- `run-lifecycle`: Postgres-backed runs, ownership checks, `POST /api/runs`, `GET /api/runs/{id}`, SSE replay with `Last-Event-ID`, event sequence de-duplication, and public-only event payloads.
- `ghost-engine`: real generated and shrunk results, recorded shrink trace, provenance, and Bug Autopsy data. The engine calls results **minimized counterexamples**, never proven global minima.
- `auth`, `grimoire-packs`, `judge-sandbox`, `language-drivers`, `hints`, and `efficiency-familiar`: their APIs and trusted execution seams are consumed here when available.

Until the upstream APIs exist, an explicitly labelled prototype route may use design fixtures for visual work. Fixtures must never be presented as real execution results and must not be used by the production workspace route or by end-to-end acceptance tests.

## Scope constraints

- Keep the canonical layout and package names. No React, Next.js, Pyodide, browser execution, alternate editor, second frontend, or client-side judge.
- Launch languages are Python and JavaScript. Java and C++ appear only when their stretch drivers are enabled by the server's language catalogue; the UI must not show unavailable languages.
- The trusted judge owns tests, expected outputs, comparison, deadlines, timing, verdicts, ghost provenance, and measured Familiar facts. The browser only displays server data and submits source text.
- The browser never receives hidden test inputs or hidden expected outputs. Hidden test rows show only public verdict, time and ordinal.
- Program output, stderr, compiler output, test values, ghost values, and AI text are rendered as plain text or structured text, never as executable HTML.
- The Familiar mood is authoritative only when received from measured `familiar`/`complexity` run events or the terminal result. There is no live typing complexity guess in this spec.
- GSAP is used only for the Familiar's measured mood animation in this workspace. Ghost reveal and shrink playback use short CSS transitions so the IDE remains calm. No ambient IDE animation, looping editor effects, or fabricated result animation.
- Use semantic Steel Night tokens from `docs/palette.md`: moss and candle for general UI, bright green only for pass/success, blood red only for failures/errors, no purple. Use Geist for UI and JetBrains Mono for code and values.
- The production workspace is tested against the actual Axum API and persisted Postgres run lifecycle. Mocked success responses are not acceptable for the definition-of-done path.

## Personas and primary journey

### P1 Learner
A job-seeker or student opens a published problem, edits a Python or JavaScript function, runs samples, submits against hidden tests, studies a minimized counterexample, fixes the draft, and submits again.

### P2 Judge
A hackathon judge signs in, opens a known problem, submits an intentionally wrong solution, sees real per-test progress and Ghost Hunting, reads the real Bug Autopsy, fixes the code, and reaches Accepted without reloading.

### P3 Keyboard and reduced-motion learner
A learner uses the same workspace without a mouse, with visible focus and non-colour status labels, or with `prefers-reduced-motion: reduce` enabled.

## EARS acceptance criteria

### R1. Workspace entry and problem context

1. WHEN an authenticated learner opens a published problem workspace, THE SYSTEM SHALL fetch the problem detail from `GET /api/problems/{slug}` and the enabled language list from `GET /api/languages` rather than embedding production problem data in the bundle.
2. THE workspace SHALL show the problem title, track/concept context, difficulty, statement, constraints, examples, expected function signature, and the currently selected language's starter code.
3. THE workspace SHALL show only languages returned as enabled by the API. Python and JavaScript SHALL be available at launch; Java and C++ SHALL be hidden or clearly marked unavailable unless their drivers ship.
4. IF the problem fetch is loading, THE SYSTEM SHALL show a calm skeleton with a text status and preserve the route identity. IF it fails, THE SYSTEM SHALL show a retry action and a safe error message without entering the editor with fabricated problem data.
5. IF the problem is missing, unpublished, or not owned by the current catalogue, THE SYSTEM SHALL show a not-found state and SHALL NOT call `POST /api/runs`.
6. The problem statement renderer SHALL allow only the supported safe markdown subset and SHALL not execute HTML or scripts from statement content.

### R2. Editor and language-specific drafts (IDE-1, IDE-4)

1. THE SYSTEM SHALL provide a Monaco editor configured for the selected language with the custom `Crypt` theme from `docs/palette.md`, syntax highlighting, line numbers, selection, undo/redo, find, accessible labels, and a visible focus treatment.
2. WHEN the learner switches between enabled languages, THE SYSTEM SHALL keep a separate draft for each `(user, problem, language)` and SHALL never replace one language's draft with another's.
3. THE SYSTEM SHALL initialise a missing draft from the API starter code and SHALL autosave edits locally with a debounced write. The local key SHALL include the problem slug and language and SHALL not include secrets or hidden test data.
4. THE SYSTEM SHALL restore a valid local draft after a reload and SHALL make the restored state explicit to assistive technology or visible status text.
5. THE SYSTEM SHALL handle quota or storage failure without losing the in-memory draft. It SHALL show a non-blocking save warning and offer copy/download of the current source as plain text.
6. THE editor SHALL enforce the API's byte limit in the UI as an early warning, but server validation SHALL remain authoritative. The UI SHALL report bytes, not line count, and SHALL not use keyword blocklists.
7. Monaco SHALL be lazy-loaded only when the workspace is entered or the editor region is opened. The initial application shell SHALL not load Monaco, GSAP, or heavy Familiar art unnecessarily.

### R3. Sample Run (IDE-2)

1. WHEN the learner presses Run, THE SYSTEM SHALL submit the current draft with `{problem, lang, code, mode: "run"}` to the real `POST /api/runs` endpoint using an idempotency key for that user action.
2. THE SYSTEM SHALL open the run SSE stream using `GET /api/runs/{id}/events` and display sample tests as they finish.
3. For sample tests, THE SYSTEM SHALL show input, expected output, actual output, verdict, and measured time. Values and output SHALL be displayed as plain text or structured text and SHALL never be interpreted as HTML.
4. THE UI SHALL distinguish queued, compiling, running, completed, failed, expired, and internal-error states with text and icons in addition to colour.
5. IF the request is rejected for validation, authentication, rate limiting, queue capacity, or an API error, THE SYSTEM SHALL show the stable error code and safe message returned by Axum, preserve the draft, and allow a retry when retrying is safe.
6. A Run SHALL not start a ghost hunt solely because a sample run failed unless the server marks and streams the corresponding ghost lifecycle; the browser SHALL follow events, not infer hidden work.

### R4. Submit and live execution (IDE-3, IDE-6)

1. WHEN the learner presses Submit, THE SYSTEM SHALL send the current source to the real `POST /api/runs` endpoint with `mode: "submit"`, then watch the persisted SSE stream.
2. THE SYSTEM SHALL show an explicit queued/claimed/compiling/running status before the first test result, including a retry or reconnect affordance when the stream is interrupted.
3. For hidden tests, THE SYSTEM SHALL display only ordinal, verdict, and time. It SHALL never display hidden input, expected output, or hidden stdout/stderr.
4. THE client SHALL apply events by `seq`, ignore duplicates and stale events, detect gaps, reconnect with the last accepted sequence, and fetch `GET /api/runs/{id}` after a terminal event. It SHALL not treat a `NOTIFY` or a client timer as an event.
5. The event client SHALL support `Last-Event-ID` replay and the query-parameter fallback used by the run-lifecycle API for non-browser clients.
6. THE UI SHALL display the final verdict using the server's authoritative terminal result. It SHALL support Accepted, Wrong Answer, Time Limit, Memory Limit, Runtime Error, Output Limit, Compile Error, expired, cancelled, and internal-error states with text and icons.
7. IF the sandbox or judge reports an infrastructure error, THE UI SHALL identify it as an internal error and SHALL not label it Wrong Answer.
8. WHEN a run is retried by the worker, THE SYSTEM SHALL show the `restarted` event and clear transient per-attempt test, ghost, bench, and Familiar state while retaining the sequence cursor.
9. A terminal run SHALL remain viewable after a page reload through `GET /api/runs/{id}`. A stream disconnect SHALL never force a full-page reload.

### R5. Ghost Hunting transition (GHO-1, GHO-4)

1. WHEN the server emits `ghost_hunting`, THE SYSTEM SHALL transition the result panel to a labelled **Ghost Hunting** state and explain that the trusted server is searching for a failing input.
2. THE browser SHALL not invent progress percentages, candidate counts, shrink steps, outputs, or timings that were not received from the server.
3. IF the server returns a minimized counterexample, THE SYSTEM SHALL show that wording and SHALL not say “smallest possible,” “globally minimal,” or “optimal.”
4. IF the engine reports that no failing input was found within budget, THE SYSTEM SHALL show that honest state, the reason if public, and any permitted fallback sample. It SHALL not imply that the learner's solution is correct.
5. IF the server aborts because of an infrastructure or unreliable-reference problem, THE SYSTEM SHALL show a safe retry/error state and SHALL not display a ghost.

### R6. Visible shrink playback (GHO-5)

1. WHEN the server emits a `shrink_step` event or supplies the persisted shrink trace, THE SYSTEM SHALL render only the recorded input, size data, and step metadata from that event/response.
2. THE playback SHALL start at the first recorded failing input and proceed through the accepted steps in sequence to the final minimized counterexample. It SHALL never create interpolated, guessed, or placeholder inputs between steps.
3. The normal playback SHALL use a single calm 150–250 ms crossfade per recorded replacement, removed content fading to ash, and a total display cap of about 3 seconds. It SHALL play once and never loop.
4. THE UI SHALL cap visible entries according to the server trace cap. If the trace is head/tail summarised, it SHALL show the server's omitted-step count and SHALL not pretend that omitted steps were displayed.
5. WHERE `prefers-reduced-motion: reduce` is active, THE SYSTEM SHALL show the same recorded steps as a static ordered list and SHALL not run CSS transitions, GSAP timelines, shakes, bursts, or blur transitions.
6. IF a shrink event arrives out of order, is duplicated, or has a sequence gap, THE reducer SHALL apply the SSE ordering rules and SHALL not play it until the missing event is replayed.

### R7. Bug Autopsy (GHO-1, GHO-2, GHO-6, VAL-5)

1. WHEN a ghost is available, THE SYSTEM SHALL show a Bug Autopsy card containing the failing input, expected output, actual output/result, failure kind, and the one-line explanation supplied by the trusted engine or server.
2. THE card SHALL render expected output in a success-labelled presentation and actual output in a failure-labelled presentation, while also using text labels/icons so status is not conveyed by colour alone.
3. THE card SHALL show structural differences from the engine, capped and marked truncated when applicable. It SHALL not recompute a different diff in the browser that could contradict the trusted data.
4. WHEN the ghost carries an authored known-ghost explanation, THE SYSTEM SHALL label it **Authored explanation**. Otherwise, when the server supplies an AI explanation, THE SYSTEM SHALL label it **AI hint**. The browser SHALL not imply an authored explanation when one was not received.
5. THE card SHALL not expose any hidden test data. A ghost is displayable only because it is a generated/public counterexample returned by the trusted judge.
6. For crash and timeout ghosts, THE card SHALL use the engine's actual result shape. A timeout SHALL be described as a slow input found and SHALL not be presented as a minimized counterexample when the engine says it was not shrunk.
7. IF the ghost payload is incomplete or fails schema validation, THE SYSTEM SHALL show a safe unavailable state and SHALL not render guessed expected/actual values.

### R8. Custom ghost rerun and authored hints

1. WHEN the learner chooses **Rerun ghost**, THE SYSTEM SHALL send the current draft and the stored ghost input through the server's dedicated custom-input run contract, with ownership and schema validation enforced server-side.
2. THE custom run SHALL show one public test with input, expected, actual, verdict, and timing because the input is the generated ghost, not a hidden test. The UI SHALL label it **Custom ghost test**.
3. THE SYSTEM SHALL never execute the current source in the browser or on the developer's machine. The custom run still uses the trusted judge sandbox and comparator.
4. THE hints panel SHALL reveal authored hints in order from nudge to targeted fix. It SHALL not skip ahead or show later hints without the learner requesting them.
5. WHEN an authored hint request fails, THE SYSTEM SHALL preserve already revealed hints and show a retry state without fabricating hint text.
6. AI hint text, where enabled by the upstream hints API, SHALL be labelled **AI hint**, grounded on the ghost and code by the server, and constrained to not provide a full solution. The browser SHALL display the returned text as plain text.

### R9. Correction and successful resolution

1. WHEN the learner edits the draft after a failed run, THE SYSTEM SHALL preserve the failed run's ghost/autopsy in the result history while associating new events with the new run id.
2. WHEN the learner resubmits, THE SYSTEM SHALL reset only the active run panel and SHALL not erase the previous autopsy until the new run has a terminal result.
3. WHEN the new submission is Accepted, THE SYSTEM SHALL show the final pass count, score, runtime, and any public timing/complexity facts supplied by the server.
4. THE SYSTEM SHALL not show a positive Familiar mood before the corresponding measured event or terminal result exists.
5. THE learner SHALL reach the Accepted result and Familiar update in the same SPA session without a page reload or manual intervention.
6. A successful terminal run SHALL trigger the existing progress/dashboard invalidation mechanism. The workspace may show a compact “progress updated” link/status, but it SHALL not claim dashboard values that were not returned by an API.

### R10. Efficiency Familiar (FAM-1, FAM-3–FAM-6)

1. THE Familiar SHALL be a docked jack-o'-lantern panel with five measured moods: Furious, Angry, Uneasy, Pleased, Laughing, using the visual rules in `docs/system-design.md` §8b.
2. THE mood SHALL update only from a server `familiar` event or authoritative terminal result containing measured correctness/scaling/confidence data. It SHALL never react to keystrokes or a static code heuristic.
3. The UI SHALL use observational wording such as “looks quadratic” and SHALL never claim “optimal.” The top state SHALL be labelled “Matches reference complexity” when the server reports that measured condition.
4. The Familiar SHALL provide mute and hide controls, persist preferences through `PATCH /api/me/familiar`, and keep a text equivalent when hidden or muted.
5. THE Familiar SHALL expose a “How does it judge me?” explanation covering medians, growth-size ratios, fit agreement, confidence, same-language baselines, shared-host noise, and the limitation that some problems have no meaningful bench curve.
6. WHEN the server finishes a run, THE UI SHALL request at most one `POST /api/runs/{id}/familiar-hint` for that run, only after the terminal result is known. The speech bubble SHALL be labelled **AI hint**.
7. IF the AI hint request fails, times out, is capped, or has no provider available, THE Familiar SHALL show the server/template fallback or a safe unavailable message. It SHALL never invent a measured class or change the mood.
8. On a failure or timeout, the Familiar may show a measured failure reaction. Under reduced motion it SHALL show a static text badge instead of crack particles or shakes.

### R11. Responsive layout and interaction

1. On a wide desktop, THE workspace SHALL present a clear statement region, editor region, and result/test region without covering editor text. The exact split may adapt to viewport width but the three responsibilities SHALL remain discoverable.
2. On a tablet-width viewport, THE workspace SHALL collapse the statement and results into keyboard-accessible drawers or tabs while keeping the editor and primary Run/Submit controls available.
3. On narrow widths, THE workspace SHALL use a single-column order: problem context, editor, run controls/status, tests, ghost/autopsy, hints, Familiar. It SHALL not require horizontal scrolling for core actions.
4. The result panel SHALL be resizable or collapsible on desktop and its state SHALL not be persisted with code drafts.
5. The layout SHALL work at 200% browser zoom and with touch targets of at least 44 CSS pixels for primary controls.
6. Heavy editor and animation assets SHALL be lazy-loaded, and the shell SHALL remain usable on a mid-range laptop and a tablet while those assets load.

### R12. Keyboard and accessibility

1. Every interactive control SHALL have an accessible name, visible focus ring, and keyboard operation. Focus SHALL use the candle `--focus` token and SHALL not disappear against the dark theme.
2. The core keyboard path SHALL support: focus problem navigation, focus editor, Run, Submit, switch result tabs, expand Ghost Hunting/Autopsy/Hints, reveal the next hint, Rerun ghost, and return focus to the editor.
3. Suggested shortcuts SHALL be `Ctrl/Cmd+Enter` for Run and `Ctrl/Cmd+Shift+Enter` for Submit, with a visible shortcut help surface. Shortcuts SHALL be disabled while focus is in a text field unless the command is intentionally editor-safe, and SHALL not override Monaco's normal editing commands unexpectedly.
4. The editor SHALL expose a programmatic label and a live status region for save state, selected language, and run state.
5. The result status SHALL use text and icons in addition to semantic colour. It SHALL expose `aria-live="polite"` updates for queued/running/test/ghost/terminal changes without flooding one announcement per animation frame.
6. The shrink trace SHALL be available as an ordered static list to screen readers. Reduced motion SHALL not remove any step or result.
7. The Familiar SHALL expose an `aria-live="polite"` text equivalent such as “Familiar: pleased, looks linear, medium confidence, reference linear.”
8. The interface SHALL preserve logical focus after async updates, reconnects, drawer changes, and modal close. It SHALL not steal focus when a background test event arrives.
9. The UI SHALL meet WCAG 2.2 AA contrast targets from `docs/palette.md`, and no meaning SHALL depend on colour, motion, sound, or hover alone.

### R13. API, security, and data handling

1. Every user-scoped read and run/event request SHALL rely on the authenticated cookie session and server ownership checks. The browser SHALL treat a missing/foreign run as the same safe not-found response.
2. The UI SHALL send state-changing requests with the API's expected JSON content type and CSRF/session requirements. It SHALL not expose session cookies or provider secrets to JavaScript.
3. The frontend SHALL cap displayed stdout/stderr/test text to the server-provided/display cap and render it as text nodes. It SHALL not use unsafe HTML injection for program output, ghost values, AI hints, or compiler messages.
4. The UI SHALL not log source code, hidden data, full outputs, cookies, or AI prompt contents to the console or analytics.
5. The UI SHALL handle stable API errors (`code`, `message`, optional retry time) without showing SQL, stack traces, worker ids, secrets, or hidden values.
6. The browser SHALL not use Valkey, Postgres, Docker, or a direct judge connection. All execution and persistence go through Axum.

### R14. Prototype and fixture integrity

1. Any fixture-backed visual route SHALL be isolated under an explicit prototype path such as `/prototype/ide-workspace` and SHALL display a persistent **Prototype fixture — not a real run** banner.
2. Fixture objects SHALL use names that cannot be confused with production run ids and SHALL not be sent to production endpoints.
3. The production `/problems/[slug]` workspace SHALL fail closed when required API data is absent; it SHALL not silently fall back to fixtures.
4. Browser acceptance tests for the definition-of-done path SHALL connect to the built SvelteKit app served by Axum and the actual API/judge stack. Test doubles may be used only for isolated reducer/component tests.

### R15. Performance and resilience

1. The first workspace shell SHALL remain responsive while Monaco and Familiar animation assets load. Monaco and GSAP SHALL be dynamically imported and disposed when the route is left; ghost CSS transitions SHALL not block the shell.
2. The client SHALL cancel or ignore stale problem/run requests when the learner navigates to another problem, so old events cannot update the new workspace.
3. The workspace SHALL tolerate SSE reconnects, temporary API errors, duplicate events, a run retry, and a terminal `204` without losing the draft or displaying contradictory results.
4. The workspace SHALL not create more than one familiar-hint request per finished run, even across component remounts or SSE reconnects.
5. The workspace SHALL expose enough timing marks for a browser performance check: shell interactive, editor ready, first event, terminal result, and total ghost reveal duration. These marks SHALL not contain source or user identifiers.

## Traceability

| Source requirement | Covered by |
|---|---|
| IDE-1 | R1, R2 |
| IDE-2 | R3 |
| IDE-3 | R4, R5, R15 |
| IDE-4 | R2 |
| IDE-5 | R11, R12 |
| IDE-6 | R4 |
| GHO-1 | R5, R7 |
| GHO-2 | R7 |
| GHO-3 | R8 |
| GHO-4 | R5 |
| GHO-5 | R6 |
| GHO-6 | R7 |
| FAM-1 | R9, R10 |
| FAM-3 | R10 |
| FAM-4 | R6, R10, R12 |
| FAM-5 | R10 |
| FAM-6 | R10 |
| VAL-5 | R3, R4, R7, R8, R13 |
| system-design §5 | R3–R9, R13, R15 |
| system-design §8 | R5–R8 |
| system-design §8b | R6, R9, R10, R12 |
| system-design §9 | R1, R3, R4, R8, R10, R13 |
| system-design §16 / palette | R1, R2, R6, R10–R12 |

## Out of scope

- Live typing complexity guesses or client-side AST heuristics.
- Java/C++ implementation before their stretch drivers are enabled.
- Browser execution, Pyodide, client-side judging, hidden-test access, or keyword/API blocklists.
- Auth implementation, OAuth/provider linking, dashboard implementation, FSRS, timed assessments, break room, payments, and MCP implementation.
- Authoring the ten problem packs or implementing the ghost engine, sandbox, language drivers, queue/lease worker, or full hint/familiar backend. This spec consumes those seams and adds only the minimum API adapters needed to make the workspace journey testable.
- Ambient IDE animation, Lottie, sound design, light mode, or a second visual theme.
