# Production Foundations Tasks

- [x] **PF01 Diagnose and test unknown static and route status behaviour**
- [x] **PF02 Return honest 404 responses without breaking valid Svelte routes**
- [x] **PF03 Publish sitemap, web manifest and security.txt**
- [x] **PF04 Make health and readiness monitoring unambiguous**
- [x] **PF05 Add privacy-safe public catalogue revalidation**
- [ ] **PF06 Repair `www` TLS and redirect to the apex [HUMAN if DNS or edge access is required]**
- [x] **PF07 Inventory CSP violations in report-only mode**
- [ ] **PF08 Enforce the stable CSP directives with rollback protection** (stage 0 live on production via #112; stage 1 merged but needs an Ansible `caddy_site` re-run to reach the shared-host edge; script enforcement still needs the Q17 decision)
- [ ] **PF09 Run router, edge, browser and live production verification**

Each task uses diagnosis where applicable, TDD, verification, review, CI, merge, cleanup, deploy and live verification before the next task starts.
