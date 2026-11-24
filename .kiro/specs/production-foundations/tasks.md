# Production Foundations Tasks

- [x] **PF01 Diagnose and test unknown static and route status behaviour**
- [x] **PF02 Return honest 404 responses without breaking valid Svelte routes**
- [x] **PF03 Publish sitemap, web manifest and security.txt**
- [x] **PF04 Make health and readiness monitoring unambiguous**
- [x] **PF05 Add privacy-safe public catalogue revalidation**
- [x] **PF06 Repair `www` TLS and redirect to the apex** (live 2026-11-24 via #115 + `caddy_site` apply: `https://www.counterspel.online/` completes TLS and 301s to apex with path+query preserved and the R6.3 header set; Evorayn health unchanged)
- [x] **PF07 Inventory CSP violations in report-only mode**
- [x] **PF08 Enforce the stable CSP directives with rollback protection** (stage 0 + stage 1 live on production 2026-11-24 via #112/#113 + `caddy_site` role apply; enforced `Content-Security-Policy` + report-only script policy + `report-uri` verified on `https://counterspel.online`, 0 browser console errors on `/problems`, Evorayn health unchanged; `script-src`/`default-src` enforcement remains a separate stage pending the Q17 decision)
- [x] **PF09 Run router, edge, browser and live production verification** (sweep recorded in `verification-2026-11-24.md`: apex/www TLS + redirects, status codes, content types, cache headers + revalidation, staged CSP enforcement, browser console, Evorayn neighbour check; found and fixed the HEAD-vs-GET cache-header divergence)

Each task uses diagnosis where applicable, TDD, verification, review, CI, merge, cleanup, deploy and live verification before the next task starts.
