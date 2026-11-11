# Licence rules for Counterspell (for Kiro)
*Checked 11 Nov 2026 on the official Kiroween rules page and each project's official repo or registry. Full evidence: research note `licence-and-llm-check-2026-11-11.md`.*

## What the hackathon requires
- **Rules text:** "The repository must be public and open source by including an open source license file. This license should be detectable and visible at the top of the repository page (in the About section)." Our repo uses **MIT** (`LICENSE` at the root).
- **Third-party SDKs, APIs and data:** "If a Project integrates any third-party SDK, APIs and/or data, Entrant must be authorized to use them in accordance with any terms and conditions or licensing requirements of the tool."
- **Dependencies:** they **don't** need to be MIT, but they must be used in line with their own licences, and pre-existing code or templates must be disclosed in the README.
- **Judges:** they must be able to test free of charge until **Fri 29 Jan 2027 23:00 SAST**.

## Allowed dependency licences
MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, 0BSD, CC0-1.0, MPL-2.0 (as a dependency only), and OFL-1.1 (fonts).
- **Accepted exception:** **GSAP** (Webflow's proprietary "Standard No Charge" licence, which allows commercial use; we must not build a visual no-code animation builder that competes with Webflow). Use it as an npm dependency only, never vendor its source, and keep its notices.

## Never add
- Stockfish or chessground (GPL);
- Judge0 (GPL);
- any GPL or AGPL library linked or copied into our code;
- code from `lichess-org/database` (AGPL; only the **CC0 puzzle CSV** is allowed);
- `rs-fsrs` (non-standard licence; use the `fsrs` crate, BSD-3, or our own scheduler);
- the deprecated `openjdk` Docker image (use `eclipse-temurin`).

## Run-only tools (fine; not distributed, not linked)
CPython (PSF), Node.js (MIT), OpenJDK/Temurin (GPL-2.0 + Classpath Exception), GCC (GPL-3.0 + Runtime Library Exception), Docker Engine/Compose, Caddy, Postgres, Valkey.

## Enforcement
- CI runs `cargo deny check licenses` with the allow-list above.
- `THIRD-PARTY-NOTICES.md` lists the Apache-2.0 NOTICE texts, the BSD notices, the OFL font notices and GSAP.
