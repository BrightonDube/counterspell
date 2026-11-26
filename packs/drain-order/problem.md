# The Drain Order

## Task
Queued events arrive as flat pairs: `events[2i]` is the urgency of event `i` and `events[2i+1]` its id. The queue always pops the most urgent event — the lowest urgency — and ties pop the lowest id. Call `drain_order(events)` and return the ids in pop order.

## Input
`events` is a flat list of urgency,id pairs; every urgency and id sits in `0..99`. An empty list drains to an empty order.

## Output
The event ids in pop order: lowest urgency first, lowest id on ties.

## Examples
- `drain_order([3, 10, 1, 20, 2, 30])` returns `[20, 30, 10]` — urgency 1, then 2, then 3.
- `drain_order([5, 42, 5, 7])` returns `[7, 42]` — the tie pops the lower id.
- `drain_order([])` returns `[]`.

## Constraints
- `0 <= len(events) <= 40`, always even; `0 <= events[i] <= 99`
