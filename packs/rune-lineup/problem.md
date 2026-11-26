# The Rune Lineup

## Task
The candles must stand in a row where no two neighbours share a mark. Call `lineups(marks)` and return the number of row orders that work — the candles are distinct even when their marks repeat.

## Input
`marks` holds the marks, one per candle.

## Output
The number of orderings of the candles with no equal neighbours.

## Examples
- `lineups([1, 2, 3])` returns `6`.
- `lineups([1, 1, 2])` returns `2` — the two `1` candles are distinct, so `1a, 2, 1b` and `1b, 2, 1a` both work.
- `lineups([1, 1])` returns `0`.

## Constraints
- `0 <= len(marks) <= 8`, `0 <= marks[i] <= 20`
