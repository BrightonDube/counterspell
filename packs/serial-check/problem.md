# The Serial Check

## Task
A schedule interleaves each rite's steps in time order — and a schedule is serial only when every rite's steps sit in one unbroken stretch. Call `serial_check(events)` and return `1` for a serial schedule, `0` for an interleaved one.

## Input
`events` lists rite ids in time order, each `0` to `12`.

## Output
`1` when no rite resumes after another rite has run, `0` otherwise.

## Examples
- `serial_check([1, 2, 1])` returns `0` — rite `1` resumes after `2`.
- `serial_check([1, 1, 2, 2, 3])` returns `1`.
- `serial_check([])` returns `1`.

## Constraints
- `0 <= len(events) <= 24`, each `0..12`
