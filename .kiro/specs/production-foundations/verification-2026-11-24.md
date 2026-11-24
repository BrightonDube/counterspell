# PF09 production verification — 2026-11-24

Live sweep of `https://counterspel.online` after PF01–PF08 all landed. Deployed
release `288aa172` (+ edge apply of `e6c9d315`'s `www` block via `caddy_site`).

## Hostnames, TLS and redirects

| Probe | Result |
|---|---|
| `https://counterspel.online/` | 200 `text/html` |
| `http://counterspel.online/` | 308 → `https://counterspel.online/` |
| `https://www.counterspel.online/` | TLS completes, 301 → `https://counterspel.online/` |
| `https://www.counterspel.online/problems?x=1` | 301 → `https://counterspel.online/problems?x=1` (path+query preserved) |
| `http://www.counterspel.online/` | 308 → `https://www.` → 301 → apex (2 hops) |
| `www` 301 headers | HSTS, nosniff, Referrer-Policy, Permissions-Policy, X-Frame-Options, CSP pair, no `Server` (R6.3 parity) |

## Status codes and content types

| Probe | Result |
|---|---|
| `GET /` | 200 `text/html` (prerendered landing) |
| `GET /problems`, `/login` | 200 `text/html` |
| `GET /definitely-not-a-route` | 404 `text/html` (honest 404 page, not SPA-200) |
| `GET /_app/immutable/does-not-exist.js` | 404 (not the HTML fallback) |
| `GET /sitemap.xml` | 200 `application/xml` |
| `GET /manifest.webmanifest` | 200 `application/manifest+json` |
| `GET /.well-known/security.txt` | 200 `text/plain` |
| `GET /api/health` | 200 `application/json` `{"status":"ok"}` |
| `GET /api/ready` | 404 at the edge (deploy-internal by design, R6.6) |

## Cache headers and revalidation

| Probe | Result |
|---|---|
| `GET /api/problems` (anonymous) | 200 `public, max-age=300` + `Vary: Cookie` + weak `ETag` |
| `GET /api/languages` (anonymous) | 200 `public, max-age=300` + `Vary: Cookie` + weak `ETag` |
| `If-None-Match` (tag, list, `*`) | 304 |
| `GET /api/problems` with session cookie | `no-store` |
| `GET /api/health` | `no-store` |
| **`HEAD /api/languages` before this PR's fix** | `no-store` — dishonest vs GET's `public` policy |
| **`HEAD /api/languages` after fix** | `public, max-age=300` + `ETag` (mirrors GET, RFC 9110 §9.3.2) |

Defect found by this sweep and fixed here: `cache_policy` matched `Method::GET`
only, so HEAD probes (curl `-I`, cache validators) saw `no-store` on resources
that are actually shared-cacheable. `head()` test helper + contract test added.

## CSP (enforced, staged)

| Probe | Result |
|---|---|
| `GET /` headers | enforcing `Content-Security-Policy` (style/img/font+data/connect/worker/media/frame/object/manifest/frame-ancestors/base-uri/form-action + `report-uri /api/csp-report`) AND `Content-Security-Policy-Report-Only` (`default-src`/`script-src` + `report-uri`) |
| Browser probe: external `img`/`fetch` | `securitypolicyviolation` with `disposition: "enforce"` AND `"report"` — both headers fire, resource blocked |
| Browser probe: inline `<script>` | `disposition: "report"` only, script executed (stage boundary working) |
| `POST /api/csp-report` | 204 (accepts legacy + report-to shapes); prod log shows real browser `script-src-elem` report from `/problems` |
| `GET /api/csp-report` | 405 |

## Browser leg (real Chromium via Playwright)

| Page | Console errors |
|---|---|
| `/` | 0 (1 warning: `GSAP target .layer-moon not found` — pre-existing cosmetic animation-layer lookup, not CSP) |
| `/problems` | 0 |

## Neighbour tenant

`https://api.evorayn.com/health` → 200 before and after each reload (the
`caddy_site` script verifies this itself; also confirmed externally).

## Edge ledger

- AUD-02 closes here: `www` TLS + 301 verified above.
- AUD-10 closed in #114 after the enforced header went live; this sweep
  re-verifies it end-to-end.
- Remaining note: `script-src`/`default-src` enforcement is a separate stage
  pending the recorded Q17-style decision (inline `200.html` bootstrap).
