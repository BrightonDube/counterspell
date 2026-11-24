# Production Foundations Tasks

- [x] **PF01 Diagnose and test unknown static and route status behaviour**
- [x] **PF02 Return honest 404 responses without breaking valid Svelte routes**
- [x] **PF03 Publish sitemap, web manifest and security.txt**
- [x] **PF04 Make health and readiness monitoring unambiguous**
- [x] **PF05 Add privacy-safe public catalogue revalidation**
- [ ] **PF06 Repair `www` TLS and redirect to the apex** (`www.counterspel.online` already resolves to 15.204.92.66; edge block + agent ansible apply via WSL — no human gate)
- [x] **PF07 Inventory CSP violations in report-only mode**
- [x] **PF08 Enforce the stable CSP directives with rollback protection** (stage 0 + stage 1 live on production 2026-11-24 via #112/#113 + `caddy_site` role apply; enforced `Content-Security-Policy` + report-only script policy + `report-uri` verified on `https://counterspel.online`, 0 browser console errors on `/problems`, Evorayn health unchanged; `script-src`/`default-src` enforcement remains a separate stage pending the Q17 decision)
- [ ] **PF09 Run router, edge, browser and live production verification**

Each task uses diagnosis where applicable, TDD, verification, review, CI, merge, cleanup, deploy and live verification before the next task starts.
