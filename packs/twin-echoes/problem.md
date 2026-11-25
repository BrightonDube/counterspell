# Twin Echoes

## Task
The vault walls echo markings. Call `repeating_echoes(echoes)` and return how many distinct markings echo at least twice, as an integer. A marking that repeats still counts once, no matter how often it echoes. An empty list echoes nothing and returns `0`.

## Input
A list of markings. Each marking is from `0` to `50`.

## Output
Return the number of distinct markings that appear at least twice, as an integer.

## Examples
`repeating_echoes([4, 1, 4, 2, 1])` returns `2`.
Marking `4` echoes and marking `1` echoes. Marking `2` does not, so the count is two.

## Constraints
The list has at most 40 markings. Each marking is from `0` to `50`.
