# The Tally Frames

## Task
A judgement frame rows predicted-against-actual tallies: the diagonal cells are the calls it got right. Call `tally_frames(m)` and return the diagonal total — every mark the frame called correctly.

## Input
`m` is a frame of tally rows; row `i`'s `i`-th cell counts calls predicted as `i` that truly were `i`.

## Output
The sum of the diagonal cells `m[i][i]`.

## Examples
- `tally_frames([[7, 2], [1, 9]])` returns `16` — `7 + 9`.
- `tally_frames([[0, 0], [0, 0]])` returns `0`.
- `tally_frames([[5]])` returns `5`.

## Constraints
- `0 <= len(m) <= 4` rows of `0..4` cells each, `0 <= m[i][j] <= 40`
