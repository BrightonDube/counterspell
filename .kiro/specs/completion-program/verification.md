# Completion Program Verification

## Static cache production evidence

Verified against `https://counterspel.online` after [deployment run 45143954536](https://github.com/BrightonDube/counterspell/actions/runs/45143954536), which deployed the merge commit from [PR #101](https://github.com/BrightonDube/counterspell/pull/101).

| Resource | HTTP 200 policy | HTTP 304 policy |
|---|---|---|
| `/_app/immutable/assets/0.C4p2jNMc.css` | `public, max-age=31536000, immutable` | same |
| `/art/creatures/skull-green-eyes.webp` | `public, max-age=604800` | same |
| `/fonts/Creepster-Regular.woff2` | `public, max-age=604800` | same |
| `/landing/moon.svg` | `public, max-age=604800` | same |
| `/robots.txt` | `public, max-age=3600` | same |

The following requests returned SPA fallback HTML without a `Cache-Control` header:

- `/_app/immutable/assets/definitely-missing.css`
- `/art/%2e%2e/robots.txt`
- `/art/%2E%2E/robots.txt`

The live verification asserted status, content type, exact cache policy and ETag-based 304 behaviour for each real resource.
