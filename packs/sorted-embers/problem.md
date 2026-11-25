# The Sorted Embers

## Task
Ember readings arrive ordered smallest first, and some readings run below zero. Call `ordered_squares(readings)` and return each reading squared, ordered smallest first, as a list. An empty list of readings returns an empty list.

## Input
A list of readings ordered smallest first; readings may be negative.

## Output
Return a list holding the square of every reading, ordered smallest first.

## Examples
`ordered_squares([-3, -1, 2])` returns `[1, 4, 9]`. Squaring flips the order of the negative readings, so the answer is rebuilt in order.

## Constraints
The list has at most 40 readings, ordered smallest first. Each reading is an integer from `-9` to `9`.
