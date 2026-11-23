# CSP Violation Inventory (PF07)

Inventory of what the current report-only policy would block if enforced verbatim,
built 2026-11-23 from static analysis and live `securitypolicyviolation` probes on
`https://counterspel.online`. Disposition `report` means the resource ran today and
would be refused under enforcement.

## Current policy

Served by the edge (`Caddyfile` and `infra/ansible/roles/caddy_site/templates/counterspell.caddy.j2`) on every response:

```
Content-Security-Policy-Report-Only:
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: blob:;
  font-src 'self';
  connect-src 'self';
  worker-src 'self' blob:;
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self'
```

**There is no `report-uri` or `report-to` today.** Violations fire only inside each
visitor's browser — nothing is collected server-side. Real-world inventory data cannot
accumulate until PF08 adds a collection endpoint.

## Live probe results (production, 2026-11-23)

Probes injected on `/` and `/problems` in a real browser; `securitypolicyviolation`
events captured with `disposition: report`:

| Probe | Violated directive | Would block under enforcement |
|---|---|---|
| Inline `<script>` element | `script-src-elem` | Yes — **this is the SvelteKit bootstrap mechanism** |
| External `<script src>` | `script-src-elem` | Yes — desired |
| Inline event handler (`onclick=`) | `script-src-attr` | Yes — desired |
| Cross-origin `fetch` | `connect-src` | Yes — desired |
| Cross-origin `<img>` | `img-src` | Yes — desired |
| Cross-origin `@font-face` font | `font-src` | Yes — desired |
| Cross-origin `<iframe>` | `frame-src` (via `default-src`) | Yes — desired |
| `style=` attribute and inline `<style>` | — | No (`style-src 'unsafe-inline'` permits) |
| `data:` image | — | No (`img-src data:` permits) |

## What the app actually does (static inventory)

| Construct | Where | Directive it needs | Covered today? |
|---|---|---|---|
| Inline `<script>` bootstrap | `web/build/200.html` (every SPA route shell) | `script-src` inline — **not covered** | **NO — the sole real conflict** |
| Inline `style="display: contents"` | `web/src/app.html` | `style-src-attr 'unsafe-inline'` | Yes (`'unsafe-inline'`) |
| Svelte-compiled event bindings | all `.svelte` | none — compiles to `addEventListener` | Yes |
| Monaco editor worker | `monaco-loader.ts` (`?worker`, Vite-emitted same-origin file) | `worker-src 'self'` | Yes |
| Guess-bubble worker | `GuessBubble.svelte` (`new Worker(new URL(...))`, same-origin) | `worker-src 'self'` | Yes (`blob:` retained for Vite-dev inlining) |
| Run-event stream | `runs/stream.ts` (`EventSource` → `/api/runs/{id}/events`) | `connect-src 'self'` | Yes |
| API/auth fetches | `lib/api/*` | `connect-src 'self'` | Yes |
| Artwork/images | `/art`, `/landing`, generated art | `img-src 'self' data: blob:` | Yes |
| `eval` / `new Function` | — not present in `src/` | `script-src 'unsafe-eval'` | Not needed |
| External origins (CDN, fonts, APIs) | — none in `src/` | — | Not needed |

## The single blocker: Q17

Every SPA-served route (`/problems`, `/dashboard`, `/login`, `/onboarding`,
`/break*`, `/settings/account`, `/prototype/*`, `/problems/*`) and the prerendered
hydration pages embed an inline bootstrap `<script>` that `script-src 'self'` will
refuse. This is the recorded **Q17 decision for Brighton** in
`docs/kiro-log/deploy-ops-blockers.md` (T08):

1. **Hash the bootstrap at build time** — emit `script-src 'self' 'sha256-…'` for the
   generated `200.html` script (and each prerendered hydration script). Strongest
   option; needs the build to publish per-file hashes to the edge config.
2. **Nonce at the edge** — Caddy injects a per-response nonce and `script-src 'nonce-…'`.
   `200.html` is a static file, so the edge must rewrite it per request — possible but
   moves cost to every HTML response.
3. **`'unsafe-inline'` on `script-src`** — simplest, but loses the main XSS mitigation
   `script-src` provides; weakens the policy's purpose.

## Proposed PF08 staging (for the decision review)

- **Stage 0** — add `report-uri`/`report-to` with a small collection endpoint so real
  traffic produces inventory data (no behaviour change).
- **Stage 1** — enforce `style-src 'self' 'unsafe-inline'`; tighten `img-src`,
  `font-src`, `connect-src`, `worker-src`, `frame-src`, `object-src 'none'`,
  `frame-ancestors`, `base-uri`, `form-action` as an enforcing `Content-Security-Policy`
  while keeping `script-src` in report-only. (No app conflict found outside `script-src`.)
- **Stage 2** — after Q17 is decided: enforce `script-src` with the chosen mechanism,
  watching reports for 48 h before calling it stable.
- **Rollback criterion** — any stage that produces a P1 regression (app fails to boot,
  workspace unusable) reverts that stage's header change only; earlier stages stay.

## Residual unknowns for live monitoring

- Signed-in routes were probed signed-out; authenticated surfaces (dashboard, settings,
  onboarding) share the same `200.html` shell, so no new violation classes are expected —
  PF08 Stage 0 reporting will confirm.
- `eval` did not produce a report-mode violation in the probe; irrelevant since no code
  path uses it.
