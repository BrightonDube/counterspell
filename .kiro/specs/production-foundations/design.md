# Production Foundations Design

## Slices

1. Correct unknown asset and route status semantics at the Axum static-serving seam.
2. Publish the required static web resources and document machine health endpoints.
3. Add privacy-safe catalogue revalidation at the public API seam.
4. Repair `www` TLS and canonical redirect at the shared edge.
5. Inventory CSP violations, remove unsafe dependencies and enforce directives in stages.

## Testing

- Router tests assert status, content type, fallback behaviour and cache isolation.
- API tests assert anonymous versus authenticated caching.
- Caddy or edge shape tests assert hostnames, redirects and security headers.
- Live probes use `curl --path-as-is`, TLS inspection and a real browser CSP issue check.
- Each CSP stage has rollback criteria and a production smoke record.

## Invariants

- No dynamic, authenticated or fallback response gains immutable caching.
- No security header is weakened to make a test pass.
- The apex remains the canonical public origin.
