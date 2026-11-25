# The Drift Alarm

## Task
A tide gauge writes a reading every hour. The watch rings a drift alarm at each hour whose reading stands strictly above every reading in the previous `width` hours. Call `drift_alarms(readings, width)` and return how many alarms ring in all.

## Input
A list of readings and a look-back width. Each reading is at most `99`, and the width is at least `1`.

## Output
Return the number of hours whose reading beats every reading in the `width` hours before it.

## Examples
- `drift_alarms([1, 3, 2, 5], 1)` returns `2`. The `3` beats the `1`; the `5` beats the `2`.
- `drift_alarms([2, 2, 2], 2)` returns `0`. Matching the look-back peak does not ring.
- `drift_alarms([5, 1, 2, 3], 3)` returns `0`. The `3` cannot top the `5` three hours back.

## Constraints
- The log holds at most 40 readings.
- Each reading is at most `99`.
- The width is between `1` and `40`.
