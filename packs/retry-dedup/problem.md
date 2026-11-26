# The Retry Dedup

## Task
A courier retries deliveries, so the log lists the same rite several times. Each row is `[seq, rite]` — the arrival order and the rite's mark. Call `retry_dedup(log)` and return how many distinct rites actually arrived.

## Input
`log` lists `[seq, rite]` rows; `seq` runs `0` to `900`, `rite` runs `0` to `900`.

## Output
The count of distinct rite marks in the log.

## Examples
- `retry_dedup([[1, 5], [2, 5], [3, 7]])` returns `2` — rite `5` arrived twice on retry.
- `retry_dedup([])` returns `0`.
- `retry_dedup([[0, 4], [1, 4], [2, 4]])` returns `1`.

## Constraints
- `0 <= len(log) <= 30` rows of `[seq, rite]` with `0 <= seq, rite <= 900`
