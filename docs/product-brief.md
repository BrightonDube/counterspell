# Counterspell: product brief (for Kiro)
> Drop this file into `docs/` and reference it from steering with `#[[file:docs/product-brief.md]]`. User stories and acceptance criteria are in `docs/requirements.md`. The full architecture is in `docs/system-design.md`, the look and feel in `docs/palette.md`, and the licence rules in `docs/licence-rules.md`. *Updated Thu 12 Nov 2026 with Brighton's decisions from the critique review (see the "Decision log 2026-11-12" in `docs/system-design.md`).*

## One line
Counterspell is a horror-themed, HackerRank-style coding practice platform. When your solution is wrong, it **hunts down an input that breaks your code and shrinks it to a minimized counterexample (the "ghost input")**, the smallest failing input it can find within a time budget, and coaches you with it.

## Who it's for
- Students and bootcamp graduates preparing for coding interviews and online assessments, including people on slow or expensive connections in Africa.
- Self-taught developers who want to practise in **Python or JavaScript** at launch (**Java and C++ are stretch goals**).

## Core loop
1. Pick a problem (tracks → concepts → problems).
2. Write code in a VS Code-style IDE (Monaco).
3. **Run** on the sample tests, or **Submit** on the hidden tests. All judging happens on our server in locked-down Docker sandboxes, and per-test results stream live. A trusted judge worker owns the tests, expected answers, comparison, time limits and the verdict; your code only prints answers.
4. On failure, the server finds a **ghost input** and shows it shrinking step by step down to the minimized counterexample. A **Bug Autopsy** card shows the input, expected vs actual, and the conceptual mistake. An authored hint ladder and an optional AI hint grounded in the ghost help you fix it.
5. The **Efficiency Familiar** (a realistic jack-o'-lantern in the IDE corner) reacts to how efficient the solution is. Its mood comes **only from measured facts** on Run/Submit (correctness and observed scaling against the reference complexity, with confidence; "looks quadratic", never "optimal"). Its speech bubble is an AI hint, labelled as such, that explains those facts and can't change the mood or verdict.
6. The dashboard updates: scores, streak, weak concepts, suggested challenges. FSRS spaced review is a stretch goal.
7. The break room (daily Lichess chess puzzle, a simple chess bot, one brain game) is a stretch goal, **built last**.

## MVP scope (deadline Fri 4 Dec 2026 14:00 PT = Sat 5 Dec 00:00 SAST; feature freeze Wed 2 Dec; submit by Fri 4 Dec 20:00 SAST)
- **Must have:** auth (GitHub, Google, email; no automatic account linking by email; a judge demo account), catalogue (**10 polished original problems**), IDE with **Python and JavaScript**, server-side sandboxed judging with live SSE results, ghost engine, hints, dashboard, landing page, Efficiency Familiar measured mood.
- **Should have:** the visible shrink animation, the Bug Autopsy card, the Kiro dogfooding demo (Kiro uses `counterspell-mcp` to find a counterexample in a deliberately buggy reference, fixes it and re-verifies), and the familiar's AI speech bubble.
- **Stretch, only after the core is freeze-safe:** Java and C++, FSRS reviews, the familiar's live-typing guess, Night Shift timed assessment, break room.
- **Out of scope:** payments, proctoring, mobile apps, copying HackerRank content.
- **Build order:** landing and visuals first, then the core, with a **hard checkpoint on Wed 25 Nov**: a real fail → ghost flow working on the VPS, or visual work pauses.

## Stack (decided by Brighton; don't change without asking)
- **Backend:** Rust (Axum) as **one binary** serving the static SvelteKit build plus the API. A separate trusted Rust **judge worker** runs code in Docker sandboxes on a **separate rootless Docker daemon used only for sandboxes**; it can't reach the main Docker daemon.
- **Data and queue:** Postgres (data, the **job queue** with `FOR UPDATE SKIP LOCKED` and leases, and persisted **run events** for SSE replay), plus Valkey for **cache and rate limits only**.
- **Hosting:** **OVH VPS-2** (4 vCPU, 8 GB, 75 GB NVMe; €8.49/month ex. VAT, VAT and backups extra), Docker Compose behind Caddy.
- **Frontend:** SvelteKit (adapter-static), Bits UI, Tailwind 4, GSAP (landing, familiar and the shrink animation only), Monaco ("Crypt" theme), chess.js + cm-chessboard (stretch).
- **Auth:** GitHub and Google OAuth plus email magic link or password, with mail via Resend.
- **AI hints:** one OpenAI-compatible client. Primary OpenAI gpt-6-luna, fallback Groq openai/gpt-oss-20b. The same client writes the familiar's speech bubble.
- **Ghost engine:** a Rust library, also exposed to Kiro as the **counterspell-mcp** server (rmcp). Its code-running tools go through the same sandbox and judge, never directly on the developer's machine.
- **Cargo workspace:** `crates/{server,judge,engine,grimoire,mcp,common}`, packages named `counterspell-<dir>`.
- **Licence:** MIT.

## Look and feel
"Steel Night":
- **Colour:** steel-blue night base, a subtle grey-green moss accent and candle amber as the second accent. Bright green is **only** for pass and red is **only** for failures. No purple.
- **Type:** Geist and JetBrains Mono, with no serif headings.
- **Motion:** cinematic on the landing page, calm in the IDE. Dark mode only at launch, using semantic tokens.
- **Assets:** realistic, scary art (not cartoon) in separate SVG files.

## People and process
- Brighton is the sole builder and **new to Rust**. Kiro writes the code and Brighton reviews it. Every Rust change must end with a short plain-English "Rust lesson".
- Hackathon: Kiroween 2026. **Main category: Costume Contest** (one category only; Frankenstein-style elements can appear in the build, but Frankenstein is not a second category). Judges score Potential Impact, Technological Implementation (how well Kiro is used), Design, Innovation and Presentation, 20% each.
- The repository must be public, MIT licensed, with `/.kiro` committed (never in `.gitignore`).
