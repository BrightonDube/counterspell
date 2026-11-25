# The Thrifty Pane

## Task
A glazier cuts panes from a strip lettered with the runes `a`, `b` and `c`. A pane is thrifty when it shows at most two different runes. Call `longest_thrifty(pane)` and return the length of the longest thrifty pane.

## Input
A string of `a`, `b` and `c` runes. It may be empty.

## Output
Return the length of the longest contiguous pane that uses at most two rune kinds.

## Examples
- `longest_thrifty("aabcc")` returns `3`. Both `aab` and `bcc` use two kinds; the whole strip uses three.
- `longest_thrifty("aaa")` returns `3`. One kind is always thrifty.
- `longest_thrifty("abcabc")` returns `2`. Every three-rune pane shows all three kinds.

## Constraints
- The strip has at most 40 runes.
- Only the runes `a`, `b` and `c` appear.
