# The Warden's Sums

## Task
A warden chalks a daily reading each day; readings are integers and may be negative. A run of days is *ritual* when its readings add up to the quota exactly. Call `ritual_spans(readings, quota)` and return how many contiguous runs are ritual, as an integer. An empty list holds no runs and returns `0`.

## Input
A list of integer readings and an integer quota.

## Output
Return the number of contiguous runs whose readings sum to the quota, as an integer.

## Examples
`ritual_spans([1, -1, 1, -1], 0)` returns `4`.
Four runs cancel to zero: days one to two, two to three, three to four, and all four days.

## Constraints
The list has at most 50 readings. Each reading is an integer from `-50` to `50`; the quota is an integer from `-500` to `500`.
