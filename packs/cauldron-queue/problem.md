# The Cauldron Line

## Task
Every mark above zero joins the back of the cauldron line; every `0` serves the mark waiting at the front (serving an empty line does nothing). Call `servings(ops)` and return how many marks were served in all.

## Input
A list of marks. Each is `0` or a positive number at most `99`.

## Output
Return the count of marks actually served.

## Examples
- `servings([2, 4, 0])` returns `1`. Two marks join; only the first is served.
- `servings([3, 0, 0])` returns `1`. The second `0` finds the line empty.
- `servings([])` returns `0`. Nothing ever happened.

## Constraints
The sequence has at most 40 entries.
