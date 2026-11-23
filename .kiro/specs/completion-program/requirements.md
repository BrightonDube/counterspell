# Counterspell Completion Program Requirements

## Purpose

Finish Counterspell as a fast, reliable, accessible, secure and memorable product. This program turns verified production and authenticated audit findings, the agreed workspace and break-room art direction, and the 200-pack Grimoire goal into ordered, independently verifiable work.

## Fixed decisions

- Keep the Steel Night visual language and existing product structure.
- Preserve decorative media. Improve performance through prioritisation, lazy loading, deferral, responsive encodes and caching, not deletion.
- Make the editor and workspace controls the critical first paint.
- Finish reliability, security, accessibility and workspace performance work before bulk catalogue publication.
- Use one focused branch and pull request at a time. Merge, clean up, deploy and verify production before starting the next slice.
- Use NeetCode's public roadmap only as taxonomy inspiration. Do not copy statements, examples, tests, hints, solutions or artwork.
- Publish 200 original Grimoire packs in 20 batches of 10, with four easy, four medium and two hard packs per batch.
- Require Python, JavaScript, Java and C++ starters before any new pack is published.
- Keep `crates/judge/` and the separate rootless sandbox daemon as the untrusted-code security boundary.
- Use the supplied Duolingo chess screenshot only as visual inspiration. Counterspell artwork and interaction must remain original.

## Program requirements

### R1. Production reliability and HTTP correctness

1. Static files use verified bounded cache policies, including 304 responses, without assigning immutable caching to SPA fallbacks or traversal requests.
2. `www.counterspel.online` has valid TLS and redirects to the apex, or its DNS record is removed so users cannot reach a broken hostname.
3. Unknown static files and unknown application routes return honest 404 responses rather than successful SPA HTML.
4. `sitemap.xml`, `manifest.webmanifest`, `security.txt` and health URLs return their documented resource or an honest 404.
5. Public catalogue reads use privacy-safe bounded caching or ETag revalidation.
6. Monitoring documentation names the exact liveness and readiness URLs and cannot treat SPA fallback HTML as a healthy service.

### R2. Security hardening

1. Content Security Policy moves from report-only to enforcement in measured stages.
2. Each stage accounts for the SvelteKit bootstrap, workers, images, fonts and required data or blob URLs.
3. Violation evidence, rollback criteria and production smoke checks are recorded before enforcement expands.
4. Existing CSRF, origin, request-size, authentication, OAuth state and sandbox protections remain intact.
5. No change exposes secrets, hidden tests, reference solutions, known ghosts or private pack data.

### R3. Language contract consistency

1. The public language list, each problem's enabled languages, starter code, editor picker, run path and judge driver agree.
2. Java and C++ are selectable only where a complete starter and working driver exist.
3. Every new pack includes complete starters for Python, JavaScript, Java and C++ before publication.
4. Contract tests prevent a globally advertised language from producing a blank editor for a published problem.

### R4. Accessibility, SEO and authenticated quality

1. Every route has an appropriate title, description and canonical metadata. Public shareable routes also have Open Graph and Twitter metadata.
2. Problem detail has one useful `h1`.
3. Chessboards have accessible names and retain keyboard-operable move input.
4. Interactive targets meet the 24 by 24 CSS-pixel minimum where applicable.
5. The dashboard has unique landmarks, and disabled onboarding controls maintain sufficient contrast.
6. Expected signed-out states do not produce misleading copy or noisy expected-401 console errors.
7. Duplicate authentication and profile requests are removed where one cached session result is sufficient.
8. Layout shifts, especially on the daily puzzle, are reduced without hiding content.

### R5. Workspace performance and costume polish

1. The problem workspace becomes interactive in under three seconds on the agreed typical mobile test profile, without editor-feature regression.
2. Problem text, editor shell and primary run, test and submit controls paint before non-critical media, Monaco workers and secondary panels.
3. Monaco languages and workers load only when needed.
4. Decorative images, videos and animation are lazy-loaded or deferred and use responsive, cached encodes with explicit dimensions.
5. Components do not all initialise or paint simultaneously when their work is not immediately visible.
6. Code editing, problem reading, controls, console, result states, hints, ghost states and Familiar feedback remain usable.
7. Costume visuals use curated existing assets, semantic palette tokens and short transform or opacity animation that respects `prefers-reduced-motion`.
8. Each workspace slice records before and after Lighthouse, bundle, transfer, interaction and layout-shift evidence.

### R6. Lantern Recall polish

1. Unrevealed tiles keep their current quiet appearance.
2. Pattern display and player selection reveal lazy-loaded themed images beneath the tiles.
3. The reveal uses a lightweight flip, fade or transparency transition rather than only a colour flash.
4. Sequence correctness, timing, level progression, score persistence and signed-out behaviour remain unchanged.
5. Roving focus, keyboard controls, announcements and reduced-motion behaviour remain correct.
6. Image loading stays outside the board's critical interaction path.

### R7. Chess experience polish

1. Bot and puzzle boards use more available width on large screens and remain safe on mobile.
2. Board squares, borders, coordinates, move highlights and states use Counterspell semantic palette tokens.
3. Pieces use original lightweight 2.5D illustration through outlines, highlights and soft shadows, without WebGL or heavy real-time 3D.
4. An original skull opponent appears above the board and uses spaced, continuous, low-cost animation during play.
5. Non-critical opponent and piece artwork is deferred or cached without delaying legal move input.
6. Bot responses, puzzle validation, board orientation, win or loss handling, score persistence and signed-out behaviour remain unchanged.
7. Chess interaction remains accessible and reduced-motion users receive a stable presentation.

### R8. Grimoire catalogue expansion

1. Add 200 original packs in 20 reviewed batches of 10.
2. Each batch contains four easy, four medium and two hard packs.
3. The program covers arrays and hashing, pointers, stacks, windows, binary search, linked structures, trees, tries, heaps, backtracking, graphs, dynamic programming, greedy methods, intervals, mathematics, geometry, bits, databases, systems, networking, distributed systems, security and AI or data processing.
4. Every pack has original prose, public samples, constraints, four authored hint rungs, a reference solution, complete visible and hidden tests, deterministic generation where appropriate and known ghosts where appropriate.
5. Each pack passes static validation, originality review, pack review and sandbox validation.
6. A batch remains unpublished if any pack fails a gate.
7. Public and private pack trees, approval hashes, retired-value checks and publication controls remain intact.

### R9. Verification and delivery

1. Every implementation slice starts with diagnosis where a defect or performance regression exists.
2. Every fix or feature follows red, green and refactor at the highest stable behavioural seam.
3. Each slice runs relevant format, lint, unit, integration, build, browser, API, accessibility, security and performance checks.
4. Each pull request receives standards and specification review before merge.
5. Required CI must pass before merge.
6. After merge, delete the feature branch and worktree, wait for deployment and verify the live behaviour at `https://counterspel.online`.
7. The final program reruns the public and authenticated audit and records all human-gated infrastructure evidence separately.

## Acceptance summary

The program is complete when all actionable child specs are implemented and live-verified, 200 original packs have passed their batch gates and are published, the final audit has no unresolved actionable P1 or P2 findings, and every remaining item is explicitly recorded as human-gated with an owner and next action.
