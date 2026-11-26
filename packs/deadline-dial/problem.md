# The Deadline Dial

## Task
Jobs arrive as flat pairs: `slots[2i]` is job `i`'s deadline and `slots[2i+1]` its duration. Each job runs to completion once started and must finish by its deadline. Call `dial_count(slots)` and return the most jobs any schedule can complete.

## Input
`slots` is a flat list of deadline,duration pairs; every value sits in `0..99`. An empty list answers `0`.

## Output
The maximum number of jobs one schedule can finish on time.

## Examples
- `dial_count([2, 1, 2, 2, 1, 3])` returns `1` — only the duration-1 job can finish by 2.
- `dial_count([3, 1, 3, 2, 2, 2])` returns `2`.
- `dial_count([])` returns `0`.

## Constraints
- `0 <= len(slots) <= 40`, always even; `0 <= slots[i] <= 99`
