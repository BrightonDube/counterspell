# Counterspell Completion Program Design

## Delivery model

The completion program is a dependency graph of focused Kiro child specs. Each child spec owns a coherent user-visible outcome and is implemented as one or more tracer-bullet pull requests. Only one implementation branch, pull request and worktree is active at a time.

The frontier order is:

1. Close the already merged static-cache slice with live evidence.
2. Repair reliability and user-safety contracts.
3. Complete language consistency.
4. Improve accessibility, SEO and authenticated quality.
5. Optimise the core workspace while preserving media.
6. Polish Lantern Recall and chess.
7. Author, review and publish the 20 catalogue batches.
8. Run the final public and authenticated verification sweep.

## Child specs

| Child spec | Outcome |
|---|---|
| `production-audit-fixes` | Own the verified audit inventory, evidence mapping and final closure gates |
| `production-foundations` | TLS, CSP, 404s, resource files, API caching and monitor-safe health semantics |
| `language-consistency` | One truthful four-language contract from catalogue to editor and judge |
| `accessibility-seo-auth-quality` | Route metadata, accessible interaction and clean authenticated or signed-out states |
| `workspace-performance-costume` | Editor-first loading under the performance target with deferred costume media |
| `lantern-recall-polish` | Image-backed tile reveals without gameplay or accessibility regression |
| `themed-chess-experience` | Responsive themed boards, lightweight 2.5D pieces and an animated skull opponent |
| `catalogue-batch-01` through `catalogue-batch-20` | Ten reviewed, sandbox-validated original packs per child spec |

## Behavioural seams

- HTTP and security changes are tested at the Axum router and deployed HTTPS response seam.
- Language consistency is tested at the public API contract and a real workspace selection and execution seam.
- Svelte route work is tested through user-visible route behaviour, accessibility scans and production bundles, not component internals.
- Lantern Recall is tested through its phase and input behaviour, keyboard interaction and saved-score API.
- Chess is tested through legal move input, bot or puzzle response, score saving and accessible SVG output.
- Packs are tested through the assembled public and private pack pipeline, the existing validator, `PackRunner`, import dry-run and publication gates.

No new broad testing abstraction is required. New seams are introduced only when an existing route or pipeline cannot express the user-visible contract.

## Workspace loading strategy

The workspace uses a staged readiness model:

1. **Shell ready:** problem text skeleton, editor surface and primary controls have stable dimensions.
2. **Editor ready:** Monaco core and the selected language are interactive.
3. **Secondary ready:** console history, hints, Familiar and result details initialise.
4. **Atmosphere ready:** decorative art and non-essential motion load during idle time or after the critical interaction.

Heavy imports remain behind dynamic boundaries. Media uses explicit dimensions, modern encodes, `loading="lazy"` where appropriate and cacheable stable URLs. Visible above-the-fold media may be eager only when measured evidence shows that it improves the user experience without delaying editor readiness.

## Lantern Recall visual model

Each tile has a stable front and a lazy image-backed reveal face. The sequence phase and player press phase both drive the same semantic reveal state. Correct and wrong feedback remains a separate state so gameplay feedback is never encoded only through the image.

Reduced motion replaces the spatial flip with an immediate or short opacity change. Images have empty alternative text because tile identity and sequence status are announced through the existing button labels and live region.

## Chess visual model

The board is sized with a responsive clamp against viewport width, available content width and viewport height. A shared themed board configuration is used by both bot and puzzle routes.

Pieces are one original SVG sprite set with vector highlights, outlines and shadows. This keeps one request, crisp scaling and no runtime 3D engine. The skull opponent uses one approved existing creature asset or an original derivative that complies with the repository licence rules. Its animation uses compositor-friendly transforms and opacity with long, irregular-feeling intervals defined by deterministic CSS timing rather than a permanent JavaScript loop.

## Catalogue batch design

Each batch contains 10 packs and has the same publication gate:

1. Draft original public and private trees.
2. Run static validation and originality review.
3. Run all four reference solutions through visible and hidden cases.
4. Prove known incorrect solutions fail where applicable.
5. Run pack review.
6. Run sandbox validation through `PackRunner`.
7. Record Brighton's approval hash.
8. Import, publish and verify the public API and one run or submit path.

Each batch uses a four easy, four medium and two hard distribution. Themes rotate so the complete programme covers foundational algorithms, advanced structures and broader computer science rather than producing 200 variations of array questions.

## Catalogue map

| Batch | Primary themes |
|---|---|
| 01 | Arrays and hashing |
| 02 | Two pointers and sequence scans |
| 03 | Stacks, queues and expression processing |
| 04 | Sliding windows and streaming statistics |
| 05 | Binary search and ordered decision spaces |
| 06 | Linked structures and pointer safety |
| 07 | Trees and binary search trees |
| 08 | Tries, strings and prefix systems |
| 09 | Heaps, priority queues and scheduling |
| 10 | Backtracking and constraint search |
| 11 | Graph traversal and connectivity |
| 12 | Weighted and advanced graphs |
| 13 | One-dimensional dynamic programming |
| 14 | Two-dimensional dynamic programming |
| 15 | Greedy methods and intervals |
| 16 | Mathematics, geometry and bit manipulation |
| 17 | Databases, SQL and data modelling |
| 18 | Operating systems, concurrency and systems |
| 19 | Networking, distributed systems and security |
| 20 | AI, data processing and mixed capstones |

## Security invariants

- Browser and server services never receive hidden tests, reference solutions or private approval files.
- Pack code runs only through the rootless sandbox boundary.
- CSP enforcement is staged and reversible.
- Cache policies never turn dynamic, private or fallback responses into public immutable content.
- No asset optimisation introduces untrusted remote dependencies.
- No implementation or documentation contains credentials.

## Verification strategy

Every child spec records exact acceptance commands and live probes. The final sweep repeats:

- public route and API correctness;
- authenticated route and API correctness;
- run, submit, ghost and score persistence where the production sandbox is available;
- Lighthouse and transfer measurements;
- axe and manual keyboard checks;
- TLS, CSP, cache, security-header and 404 probes;
- catalogue counts, difficulty, domain and language coverage;
- pack import, publication and sandbox evidence.

Human-controlled DNS, provider accounts, monitoring, encrypted off-box backup, restore rehearsal and rollback evidence remain explicit operator tasks rather than being marked complete from code alone.
