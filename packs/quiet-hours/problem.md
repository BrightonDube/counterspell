# The Quiet Hours

## Task
A watch log lists a bustle reading for every hour. A stretch of three consecutive hours is quiet when their readings add to no more than the `quota`. Call `quiet_windows(readings, quota)` and return how many three-hour stretches are quiet.

## Input
A list of hourly readings and a quota. Each reading is at most `99`.

## Output
Return the count of three-hour stretches whose sum is at most `quota`.

## Examples
- `quiet_windows([1, 2, 3, 4, 5], 6)` returns `1`. Only the first stretch, `1 + 2 + 3 = 6`, is quiet.
- `quiet_windows([0, 0, 0], 0)` returns `1`. The single stretch is perfectly still.
- `quiet_windows([5, 5], 10)` returns `0`. Two hours cannot fill a three-hour stretch.

## Constraints
- The log holds at most 40 readings.
- Each reading is at most `99`.
- The quota is between `0` and `300`.
