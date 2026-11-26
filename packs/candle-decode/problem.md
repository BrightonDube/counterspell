# The Candle Decode

## Task
A line of glyph digits decodes into letters — each single glyph `1..9` reads as a letter, and each pair `10..26` reads as one too. Call `decode_ways(glyphs)` and return how many full decodings the line carries.

## Input
`glyphs` is a string of digits.

## Output
The decoding count; an empty line decodes one way, and a line stuck on a `0` decodes none.

## Examples
- `decode_ways("")` returns `1`.
- `decode_ways("12")` returns `2` — `1|2` or `12`.
- `decode_ways("06")` returns `0` — the leading zero decodes nothing.

## Constraints
- `0 <= len(glyphs) <= 40`; every glyph is `0-9`
