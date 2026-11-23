# Production Audit Baselines (PA02)

Original public and authenticated baselines recorded before completion-program
implementation. Public probes re-run 2026-11-23 ~15:30 UTC against
`https://counterspel.online` serving main `2d4e88d1` (post-CP00). Authenticated
findings are carried from the verified audit; sandbox-dependent paths stay open
until direct production evidence exists (AUD-26).

## Public HTTP baseline

| Probe | Result | Expected after fix |
|---|---|---|
| `GET /` | `200 text/html` | unchanged |
| `GET /api/health` | `200 application/json` | unchanged (public liveness) |
| `GET /api/ready` | `404` (edge blocks by design) | unchanged, documented (AUD-08) |
| `GET /api/problems` | `200 application/json`, `Cache-Control: no-store` | bounded shared caching or ETag `304` (AUD-09) |
| `GET /api/languages` | `200 application/json`, `Cache-Control: no-store` | same (AUD-09) |
| `GET /api/nope` | `404 application/json` | unchanged (honest API 404s already correct) |
| `GET /sitemap.xml` | `200 text/html` SPA fallback | real sitemap or honest `404` (AUD-05) |
| `GET /manifest.webmanifest` | `200 text/html` SPA fallback | real manifest or honest `404` (AUD-06) |
| `GET /.well-known/security.txt` | `200 text/html` SPA fallback | real security.txt or honest `404` (AUD-07) |
| `GET /security.txt` | `200 text/html` SPA fallback | same (AUD-07) |
| `GET /robots.txt` | `200 text/plain`, `public, max-age=3600` | unchanged |
| `GET /no-such-route-xyz` | `200 text/html` SPA fallback | honest `404` (AUD-04) |
| `GET /definitely-missing-asset.png` | `200 text/html` SPA fallback | honest `404` (AUD-03) |
| `GET https://www.counterspel.online/` | TLS handshake failure | valid TLS + apex redirect, or record removed (AUD-02) |

## Security header baseline (on `/`)

Present: `Strict-Transport-Security: max-age=31536000; includeSubDomains`,
`X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`,
`Referrer-Policy: strict-origin-when-cross-origin`,
`Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=()`.

CSP is report-only, not enforced (AUD-10):

```text
Content-Security-Policy-Report-Only:
  default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
  img-src 'self' data: blob:; font-src 'self'; connect-src 'self';
  worker-src 'self' blob:; frame-ancestors 'none'; base-uri 'self';
  form-action 'self'
```

Static cache policies verified live in
`.kiro/specs/completion-program/verification.md` (AUD-01, closed post-CP00 but
kept under regression watch).

## Authenticated baseline (carried from the verified audit)

Findings below were verified in the authenticated audit and are reproduced here so
every ledger entry has a baseline reference. Live re-probes happen in each owning
spec's acceptance step.

| Finding ref | Baseline observation |
|---|---|
| AUD-11 | Language list, problem languages, starters, editor picker and judge drivers can disagree; Java and C++ selectable without a complete contract |
| AUD-12/13 | Route metadata and problem `h1` gaps in generated HTML |
| AUD-14/15/16/17/25 | axe findings: chessboard names, <24 px targets, duplicate dashboard landmarks, disabled-control contrast, unresolved surfaces needing manual review |
| AUD-18/19/20 | Signed-out copy, expected-401 console noise, duplicate auth/profile requests |
| AUD-21 | Layout shift, notably the daily puzzle |
| AUD-22/23 | Workspace interaction latency over the 3 s target; missing-GSAP-target console warning |
| AUD-26 | Run, submit, ghost and IDE paths unverified on production — open until direct evidence |

## Reproduction

```bash
for p in / /api/health /api/ready /api/problems /api/languages /api/nope \
         /sitemap.xml /manifest.webmanifest /.well-known/security.txt \
         /security.txt /robots.txt /no-such-route-xyz /definitely-missing-asset.png; do
  curl -sI --max-time 15 "https://counterspel.online$p" | head -1
done
curl -sv https://www.counterspel.online/   # TLS handshake failure (AUD-02)
curl -sI https://counterspel.online/ | grep -i content-security
curl -sI https://counterspel.online/api/problems | grep -i cache-control
```
