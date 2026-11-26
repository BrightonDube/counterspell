# The Rate Gate

## Task
A ward gate holds a bucket of `capacity` seals, refilling `refill` seals each tick up to the brim. Each event row `[tick, take]` asks for `take` seals at that tick; when the bucket holds enough the event passes and the seals are spent, otherwise it is turned away. Call `rate_gate(events, capacity, refill)` and return how many events passed.

## Input
`events` lists `[tick, take]` rows ordered by tick (`0` to `200` each); `capacity` and `refill` run `0` to `60`.

## Output
The count of admitted events. The bucket starts full at `capacity` and the clock starts at tick `0`.

## Examples
- `rate_gate([[0, 2], [1, 2], [2, 2]], 3, 1)` returns `2` — the third ask finds only `1` seal.
- `rate_gate([[0, 5]], 3, 1)` returns `0` — five seals can never come from a three-seal bucket.
- `rate_gate([[0, 4], [4, 4]], 4, 1)` returns `2` — four quiet ticks refill the brim.

## Constraints
- `0 <= len(events) <= 30` rows of `[tick, take]` with `0 <= tick, take <= 200`, ticks nondecreasing
- `0 <= capacity, refill <= 60`
