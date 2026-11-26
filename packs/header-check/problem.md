# The Header Check

## Task
Every packet entering the ward carries an envelope `[version, sender, seq, length]`. The gate must fail closed: any field outside its bound voids the packet. Call `header_check(env)` and return `1` when the envelope is sound, `0` when it must be dropped.

## Input
`env` is a list of four integers.

## Output
`1` only when `version` is `1` or `2`, `sender` runs `0` to `255`, `seq` runs `0` to `65535` and `length` runs `0` to `1500`; otherwise `0`.

## Examples
- `header_check([1, 5, 10, 100])` returns `1` — a sound envelope.
- `header_check([3, 5, 10, 100])` returns `0` — version `3` is unknown.
- `header_check([1, 5, -1, 100])` returns `0` — a negative sequence fails closed.

## Constraints
- `len(env) = 4`, each field `-1` to `70000`
