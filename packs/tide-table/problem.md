# The Tide Table

## Task
A keeper writes down the tide height every hour. Call `first_drop(heights)` and return the position of the first hour whose reading is lower than the reading before it. Positions count from `0`. Return `-1` when the tide never drops.

## Input
A list of readings in hourly order. Each reading is a height from `0` to `100`.

## Output
Return the position of the first strictly lower reading as an integer, or `-1`.

## Examples
`first_drop([3, 5, 5, 2])` returns `3`.
The readings rise, hold level, then hour `3` reads lower than the hour before it. That is the first drop.

## Constraints
The list has at most 48 readings. Each reading is from `0` to `100`.
