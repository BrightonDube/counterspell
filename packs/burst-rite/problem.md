# The Burst Rite

## Task
A single engine runs rites in the order they arrive — first come, first served, never paused mid-rite. Each rite waits from its arrival until it starts. Call `burst_rite(jobs)` and return the total waiting across all rites.

## Input
`jobs` lists `[arrival, burst]` rows; arrivals and bursts run `0` to `60`. A rite that arrives while the engine rests starts at once; ties in arrival keep their listed order.

## Output
The sum of `start - arrival` over all rites.

## Examples
- `burst_rite([[1, 4], [0, 2], [2, 3]])` returns `5` — starts `0, 2, 6`, waits `0, 1, 4`.
- `burst_rite([])` returns `0`.
- `burst_rite([[0, 5]])` returns `0`.

## Constraints
- `0 <= len(jobs) <= 16` rows of `[arrival, burst]` with `0 <= arrival, burst <= 60`
