# The Triage Mark

## Task
The ward queue holds urgency marks — lower is more urgent. Call `next_mark(prios)` and return the index of the most urgent patient; ties take the earliest index.

## Input
`prios` holds urgency marks. An empty queue answers `-1`.

## Output
The index of the smallest mark, earliest index on ties; `-1` when the queue is empty.

## Examples
- `next_mark([3, 1, 2])` returns `1`.
- `next_mark([2, 2, 5])` returns `0` — the earliest tie wins.
- `next_mark([])` returns `-1`.

## Constraints
- `0 <= len(prios) <= 40`, `0 <= prios[i] <= 99`
