# The Sched Rite

## Task
Binding rules say rite `a` must finish before rite `b` — a schedule honours them only when no rule chain circles back. Call `sched_rite(pairs)` and return `1` when the rules are acyclic, `0` when some rite must precede itself.

## Input
`pairs` lists `[a, b]` precedence rules; rite ids run `0` to `12`.

## Output
`1` for an acyclic rule set, `0` when a cycle exists.

## Examples
- `sched_rite([[1, 2], [2, 3]])` returns `1`.
- `sched_rite([[1, 2], [2, 1]])` returns `0` — each waits on the other.
- `sched_rite([])` returns `1`.

## Constraints
- `0 <= len(pairs) <= 24` rows of `[a, b]` with `0 <= a, b <= 12`
