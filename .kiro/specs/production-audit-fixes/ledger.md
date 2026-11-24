# Production Audit Fixes Ledger (PA01)

One row per verified audit finding. Severity: **P1** user-facing correctness or
security, **P2** contract or quality, **P3** polish or evidence hygiene.

Status key:

- `open` — not yet repaired.
- `owner-merged` — fix merged; live probe pending.
- `live-verified` — verified at `https://counterspel.online` with recorded evidence.
- `human-gated` — needs operator access (DNS, provider console, monitoring); owner
  and next action recorded, never marked code-complete.
- `closed` — live-verified, or explicitly accepted as human-gated evidence.

Every entry names exactly one owning child spec and one live acceptance probe, per
`design.md`. Baseline probe results are in `baselines.md` (PA02).

## Findings

| ID | Sev | Finding | Original evidence | Owner | Live acceptance probe | Status |
|---|---|---|---|---|---|---|
| AUD-01 | P2 | Static assets had no verified bounded cache policies | Public audit; fixed by PR #101 | completion-program CP00 | `completion-program/verification.md` curl matrix: immutable `31536000`, art/fonts `604800`, `robots.txt` `3600`, 304 keeps policy | live-verified |
| AUD-02 | P1 | `www.counterspel.online` has no valid TLS and no apex redirect | `curl https://www.counterspel.online` TLS handshake failure, 2026-11-23 | production-foundations PF06 | `curl -sv https://www.counterspel.online/` completes TLS and 301/302s to apex, or the DNS record is removed | closed 2026-11-24: `https://www.counterspel.online/` completes TLS and 301s to `https://counterspel.online/`; path+query preserved (`/problems?x=1` → apex same); redirect carries HSTS/nosniff/XFO/Permissions-Policy/no `Server`; `http://www` → 308→301→apex; Evorayn health 200 (#115 + `caddy_site` apply) |
| AUD-03 | P1 | Missing static files return `200 text/html` SPA fallback | `curl -I /definitely-missing-asset.png` → `200 text/html`, 2026-11-23 | production-foundations PF01/PF02 | `curl -I` on missing static paths returns honest `404`, real routes still work | closed 2026-11-23: live `curl -I` shows missing statics → 404 no-store; real assets still 200 (#104) |
| AUD-04 | P1 | Unknown application routes return `200 text/html` | `curl -I /no-such-route-xyz` → `200 text/html`, 2026-11-23 | production-foundations PF01/PF02 | `curl -I` on an unknown route returns honest `404` | closed 2026-11-23: live `curl -I /no-such-route-xyz` → 404 text/html + no-store; declared routes 200 (#104) |
| AUD-05 | P2 | `/sitemap.xml` returns SPA HTML | `curl -I /sitemap.xml` → `200 text/html`, 2026-11-23 | production-foundations PF03 | `200 application/xml` sitemap, or honest `404` | closed 2026-11-23: live `/sitemap.xml` → 200 application/xml (#105) |
| AUD-06 | P2 | `/manifest.webmanifest` returns SPA HTML | `curl -I /manifest.webmanifest` → `200 text/html`, 2026-11-23 | production-foundations PF03 | `200 application/manifest+json` (or equivalent real resource) or honest `404` | closed 2026-11-23: live `/manifest.webmanifest` → 200 application/manifest+json (#105) |
| AUD-07 | P2 | `/.well-known/security.txt` and `/security.txt` return SPA HTML | `curl -I` both paths → `200 text/html`, 2026-11-23 | production-foundations PF03 | `200 text/plain` security.txt, or honest `404` | closed 2026-11-23: live `/.well-known/security.txt` → 200 text/plain; `/security.txt` honest 404 (#105) |
| AUD-08 | P2 | Health and readiness URLs are not documented unambiguously; SPA HTML must never count as healthy | Audit; `/api/health` → `200 application/json`, `/api/ready` → edge `404` (internal by design), 2026-11-23 | production-foundations PF04 | Monitoring doc names the exact liveness/readiness URLs; external probes confirm `/api/health` JSON and `/api/ready` edge behaviour | closed 2026-11-23: docs/monitoring.md published; live `/api/health` 200 application/json, `/api/ready` edge 404 (#107) |
| AUD-09 | P2 | Public catalogue GETs send `Cache-Control: no-store` with no ETag revalidation | `curl -I /api/problems` and `/api/languages` → `no-store`, 2026-11-23 | production-foundations PF05 | Anonymous catalogue GETs carry bounded shared caching or working ETag `304` revalidation; authenticated responses stay uncached | closed 2026-11-23: live `/api/languages` and `/api/problems` → 200 `public, max-age=300` + `Vary: Cookie` + `ETag: W/"…"`; `If-None-Match` (tag and `*`) → 304; cookie-bearing and `/api/health` → `no-store` (#109) |
| AUD-10 | P1 | CSP is `Content-Security-Policy-Report-Only`, not enforced | Response headers on `/` and API GETs, 2026-11-23 | production-foundations PF07/PF08 | Staged enforcing `Content-Security-Policy` header on production with recorded violation evidence and rollback criteria | closed 2026-11-24: enforced CSP live on `https://counterspel.online` (style/img/font+data/connect/worker/media/frame/object/manifest/frame-ancestors/base-uri/form-action + `report-uri`); report-only policy retains `script-src` until Q17; live violation evidence in prod log (`script-src-elem blocked="inline" disposition="report"` from a real browser on `/problems`); 0 console errors on `/problems`; rollback = `counterspell-caddy-site` marked-block restore (validate + Evorayn health guard + auto-restore) |
| AUD-11 | P1 | Language contract may disagree across API, problem, starters, editor and judge | Authenticated audit; `GET /api/languages` vs per-problem enabled set | language-consistency | Contract tests plus a live workspace check: every advertised language has a complete starter and working driver, and selection never yields a blank editor | open |
| AUD-12 | P2 | Routes lack distinct title, description and canonical metadata; shareable routes lack OG/Twitter tags | Public audit of generated HTML | accessibility-seo-auth-quality | View-source on public routes shows the required metadata | open |
| AUD-13 | P2 | Problem detail lacks one descriptive `h1` | Public/axe audit | accessibility-seo-auth-quality | Problem route renders exactly one descriptive `h1` | open |
| AUD-14 | P2 | Chessboard SVGs lack accessible names | Authenticated axe audit | accessibility-seo-auth-quality | axe pass on chess routes; boards expose names without duplicating move announcements | open |
| AUD-15 | P2 | Back controls, checkboxes and other targets below 24×24 CSS px | Authenticated axe audit | accessibility-seo-auth-quality | axe target-size pass on affected routes | open |
| AUD-16 | P2 | Dashboard renders duplicate landmarks | Authenticated axe audit | accessibility-seo-auth-quality | axe landmark pass on `/dashboard` | open |
| AUD-17 | P2 | Disabled onboarding controls fail contrast | Authenticated axe audit | accessibility-seo-auth-quality | Contrast measured on the onboarding route | open |
| AUD-18 | P2 | Signed-out chess copy implies already signed-in users are anonymous | Authenticated audit | accessibility-seo-auth-quality | Manual route copy check signed-in and signed-out | open |
| AUD-19 | P3 | Expected signed-out states emit noisy expected-401 console errors | Authenticated audit | accessibility-seo-auth-quality | Browser console clean on signed-out routes | open |
| AUD-20 | P3 | Duplicate authentication and profile requests where one cached session result suffices | Authenticated audit | accessibility-seo-auth-quality | Network panel shows single session/profile fetches | open |
| AUD-21 | P2 | Layout shift on the daily puzzle and other unstable surfaces | Public + authenticated audit | accessibility-seo-auth-quality | Lighthouse CLS evidence before/after | open |
| AUD-22 | P1 | Workspace time-to-interactive exceeds 3 s on the agreed mobile profile; non-critical media and Monaco workers load on the critical path | Authenticated performance audit | workspace-performance-costume | Before/after Lighthouse, transfer, interaction and layout-shift evidence under the agreed profile | open |
| AUD-23 | P3 | Missing GSAP target warning in console | Authenticated audit | workspace-performance-costume | Browser console free of the GSAP target warning | open |
| AUD-24 | P2 | Production source-map exposure decision unrecorded | Public audit | production-foundations | Recorded decision: ship or strip production source maps, with rationale | open |
| AUD-25 | P2 | Surfaces axe could not resolve need a manual contrast review | Authenticated audit | accessibility-seo-auth-quality | Recorded manual contrast pass over the unresolved surfaces | open |
| AUD-26 | P1 | Production run, submit, ghost and IDE paths lack direct live evidence | Spec requirement; sandbox-dependent | production-audit-fixes PA06 | Recorded production evidence for run, submit, ghost and IDE paths | open |
| AUD-27 | P2 | Catalogue lacks the agreed 200 original packs (80 easy, 80 medium, 40 hard, four-language coverage) | Catalogue audit | catalogue-batch-01…20 | `CP22` live count and language coverage check | open |
| AUD-28 | P2 | Human-gated infrastructure items (DNS, monitoring, encrypted off-box backup, restore rehearsal, rollback) need owners | Spec requirement | production-audit-fixes PA07 | `CP23` record: every human-gated item has an owner and exact next action | human-gated |

## Notes

- AUD-01 is already live-verified (CP00); it stays in the ledger so the final audit
  re-checks it for regressions.
- AUD-02 is agent-actionable: `www.counterspel.online` already resolves to
  15.204.92.66, so the fix is a `www` redirect block in the shared-host edge applied
  via the `caddy_site` role (agent-runnable from WSL: `counterspell-vps` SSH alias +
  `~/.config/counterspell/vault-pass`).
- AUD-26 stays open until direct production evidence exists, per the spec — sandbox
  verification is never inferred from code alone.
