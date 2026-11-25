# The Three-Rune Windows

## Task
A mosaic band is painted with the runes `a`, `b` and `c`. A pane is complete when it shows all three runes at least once. Call `count_varied(inscription)` and return how many contiguous panes of the band are complete.

## Input
A string of `a`, `b` and `c` runes. It may be empty.

## Output
Return the number of contiguous substrings that contain `a`, `b` and `c` each at least once.

## Examples
- `count_varied("abc")` returns `1`. Only the whole band is complete.
- `count_varied("abcabc")` returns `10`. Four panes start at the first `a`, three at the first `b`, two at the first `c` and one at the second `a`.
- `count_varied("aaa")` returns `0`. No `b` or `c` ever appears.

## Constraints
- The band has at most 40 runes.
- Only the runes `a`, `b` and `c` appear.
