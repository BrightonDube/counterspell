# The Calm Spell

## Task
A chant is written in the runes `a`, `b` and `#`. Every `#` is a crack that splits the chant into spells. Call `longest_calm(chant)` and return the length of the longest spell — the longest stretch with no `#` inside.

## Input
A string of `a`, `b` and `#` runes. It may be empty.

## Output
Return the length of the longest stretch without a `#`, or `0` when no stretch exists.

## Examples
- `longest_calm("ab##ba")` returns `2`. The `ab` and `ba` spells are both two runes long.
- `longest_calm("###")` returns `0`. Cracks and nothing else.
- `longest_calm("ababa")` returns `5`. No crack ever lands.

## Constraints
- The chant has at most 40 runes.
- Only the runes `a`, `b` and `#` appear.
