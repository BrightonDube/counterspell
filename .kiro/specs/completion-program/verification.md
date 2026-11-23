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

## Reproduction commands

```bash
set -euo pipefail

check_cache() {
  url="$1"
  policy="$2"
  content_type="$3"
  headers="$(mktemp)"
  curl --silent --show-error --dump-header "$headers" --output /dev/null "$url"
  grep -E "^HTTP/[0-9.]+ 200" "$headers"
  tr -d '\r' < "$headers" | grep -Fxi "cache-control: $policy"
  tr -d '\r' < "$headers" | grep -Fxi "content-type: $content_type"
  etag="$(awk 'BEGIN { IGNORECASE=1 } /^etag:/ { sub(/\r$/, ""); sub(/^etag:[[:space:]]*/, ""); print; exit }' "$headers")"
  revalidation_headers="$(mktemp)"
  curl --silent --show-error --dump-header "$revalidation_headers" --output /dev/null \
    --header "If-None-Match: $etag" "$url"
  grep -E "^HTTP/[0-9.]+ 304" "$revalidation_headers"
  tr -d '\r' < "$revalidation_headers" |
    grep -Fxi "cache-control: $policy"
  rm -f "$headers" "$revalidation_headers"
}

check_cache "https://counterspel.online/_app/immutable/assets/0.C4p2jNMc.css" \
  "public, max-age=31536000, immutable" "text/css"
check_cache "https://counterspel.online/art/creatures/skull-green-eyes.webp" \
  "public, max-age=604800" "image/webp"
check_cache "https://counterspel.online/fonts/Creepster-Regular.woff2" \
  "public, max-age=604800" "font/woff2"
check_cache "https://counterspel.online/landing/moon.svg" \
  "public, max-age=604800" "image/svg+xml"
check_cache "https://counterspel.online/robots.txt" \
  "public, max-age=3600" "text/plain"

for path in \
  "/_app/immutable/assets/definitely-missing.css" \
  "/art/%2e%2e/robots.txt" \
  "/art/%2E%2E/robots.txt"
do
  headers="$(mktemp)"
  curl --silent --show-error --dump-header "$headers" --output /dev/null \
    "https://counterspel.online${path}"
  grep -E "^HTTP/[0-9.]+ 200" "$headers"
  tr -d '\r' < "$headers" | grep -Fxi "content-type: text/html"
  if grep -qi '^cache-control:' "$headers"; then
    exit 1
  fi
  rm -f "$headers"
done
```
