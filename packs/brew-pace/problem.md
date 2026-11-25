# The Brew Pace

## Task
A keeper must finish a row of brews within `hours` hours. Brewing `pace` cauldrons at once finishes brew `i` in `ceil(brews[i] / pace)` hours of its own. Call `min_pace(brews, hours)` and return the smallest pace that finishes every brew inside the deadline — the hours each brew needs are added across the row.

## Input
A list of brew sizes and the deadline in hours. The list is never empty.

## Output
The smallest integer pace such that the total `ceil(brew / pace)` hours over all brews stays `<= hours`.

## Examples
- `min_pace([3, 6, 7], 9)` returns `2` — pace `2` needs `2 + 3 + 4 = 9` hours, pace `1` needs `16`.
- `min_pace([1, 1, 1], 3)` returns `1`.
- `min_pace([9, 9], 2)` returns `9`.

## Constraints
- `1 <= len(brews) <= 40`
- brew sizes are integers, `1 <= size <= 99`
- `hours` is an integer, `len(brews) <= hours <= 4000`
