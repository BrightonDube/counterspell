# The Token Scope

## Task
A gate token lists the warding scopes it grants, and it dies at `expiry`. The rite needs every scope in `required` — missing any, or arriving after the token's last live tick, fails the check closed. Call `scope_ok(required, granted, expiry, now)` and return `1` only when the request is fully covered and the token still lives.

## Input
`required` and `granted` are scope-code lists (`0` to `30` each, lengths `0` to `8`); `expiry` runs `-1` to `1000` (negative means already dead), `now` runs `0` to `1000`.

## Output
`1` only when every required scope appears in `granted` and `now < expiry`; otherwise `0`.

## Examples
- `scope_ok([1, 2], [1, 2, 3], 10, 5)` returns `1` — covered and alive.
- `scope_ok([1, 4], [1, 2, 3], 10, 5)` returns `0` — scope `4` is missing.
- `scope_ok([1], [1], 10, 10)` returns `0` — the token dies at `10`.

## Constraints
- `0 <= len(required), len(granted) <= 8`, codes `0` to `30`; `-1 <= expiry <= 1000`; `0 <= now <= 1000`
