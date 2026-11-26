# The Quad Mark

## Task
The map is quartered: quadrant `1` holds marks right and up, `2` left and up, `3` left and down, `4` right and down. A mark on either axis holds no quadrant. Call `quad_mark(x, y)` and return the quadrant, or `0` on an axis.

## Input
`x` and `y` are integers from `-100` to `100`.

## Output
`1`–`4` inside a quadrant, `0` on an axis.

## Examples
- `quad_mark(3, 4)` returns `1`.
- `quad_mark(-3, 4)` returns `2`.
- `quad_mark(0, 5)` returns `0`.

## Constraints
- `-100 <= x, y <= 100`
