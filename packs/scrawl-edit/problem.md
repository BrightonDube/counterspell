# The Scrawl Edit

## Task
A scrawl is mended one letter at a time — swap a glyph, drop one, or insert one. Call `edit_toll(first, second)` and return the fewest mends that turn the first scrawl into the second.

## Input
`first` and `second` are the two scrawls, letters `a-z`.

## Output
The fewest mends; equal scrawls need none.

## Examples
- `edit_toll("flame", "frame")` returns `1` — swap `l` for `r`.
- `edit_toll("ember", "embers")` returns `1` — insert the `s`.
- `edit_toll("kitten", "sitting")` returns `3`.

## Constraints
- `0 <= len(first), len(second) <= 20`
