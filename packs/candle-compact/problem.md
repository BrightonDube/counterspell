# The Candle Compact

## Task
A row of candles carries sigil marks, and one cursed sigil must be purged. Call `remaining_candles(marks, cursed)` and return how many candles keep a mark other than the cursed sigil, as an integer. An empty row keeps none.

## Input
A list of sigil marks, then the cursed sigil to purge.

## Output
Return the count of candles whose mark differs from the cursed sigil, as an integer.

## Examples
`remaining_candles([1, 2, 1, 3], 1)` returns `2`. Both candles marked `1` are purged; the `2` and the `3` remain.

## Constraints
The row has at most 40 marks. Each mark and the cursed sigil are from `0` to `9`.
