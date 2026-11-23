# Production Foundations Requirements

## Goal

Make every public Counterspell hostname, response and browser security policy truthful, monitorable and safe.

## Requirements

- `www.counterspel.online` must have valid TLS and redirect to the apex, or its DNS record must be removed.
- Missing static resources and unknown routes must return an honest 404.
- `sitemap.xml`, `manifest.webmanifest` and `/.well-known/security.txt` must be real resources or honest 404s.
- Health and readiness URLs must return documented machine-readable responses and must not be confused with SPA fallback HTML.
- Public catalogue GETs may use bounded shared caching or ETag revalidation only where responses contain no user-specific data.
- CSP must progress from report-only to enforcement in reversible stages with production violation evidence.
- Existing HSTS, frame denial, nosniff, referrer, permissions, CSRF, origin and request-size protections must remain intact.
- Production verification must cover apex and `www`, TLS, redirects, status codes, content types, cache headers and CSP.

## Out of scope

- Changing the judge sandbox boundary.
- Marking DNS, monitoring, backup or restore work complete without operator evidence.
