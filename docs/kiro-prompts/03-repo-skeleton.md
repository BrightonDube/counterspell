# D.3 Spec 1: `repo-skeleton` (Feature, Design-First, Low-Level)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Create a feature spec named repo-skeleton using the Design-First workflow (low-level design), because the architecture is already decided in #[[file:docs/system-design.md]] (§2 Architecture, §3 Components, §10 Deployment, §14 first milestone).
Scope: Cargo workspace with exactly the layout in §2 (crates/server, judge, engine, grimoire, mcp, common, published as counterspell-server, counterspell-judge, counterspell-engine, counterspell-grimoire, counterspell-mcp, counterspell-common; no other crate names), Axum server serving the SvelteKit adapter-static build and /api/health, SvelteKit app in web/ with Tailwind 4 tokens from docs/palette.md, docker-compose with postgres + valkey (cache and rate limits only) + caddy, a deploy/ note for the separate rootless sandbox daemon, sqlx migrations folder, GitHub Actions CI (fmt, clippy, test, svelte-check, cargo deny licenses), MIT LICENSE, THIRD-PARTY-NOTICES.md.
Done means: `docker compose up` serves the Svelte page from the Rust binary at localhost and CI is green. Keep tasks small (≤ 30 min each) and mark which tasks can run in parallel.
```
