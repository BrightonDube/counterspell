# Author checklist

Tick each item before the S9 static check. The items are for the author agent. The reviewer uses its own checklist (`reviewer.md`).

- `AU-1` The wording is my own. I did not open or quote a third-party problem statement (PIP-3).
- `AU-2` The function uses only supported types: `int`, `str`, `list[int]` and `list[list[int]]`.
- `AU-3` Every constraint matches the input schema. Wherever negative numbers are allowed, the statement says "integer".
- `AU-4` The worked example calls the function with sample 1 and gives two to four lines of explanation.
- `AU-5` There are at least eight curated hidden cases. Each has a hand-derived expected value and a `why` note of 1 to 120 characters.
- `AU-6` The hidden cases cover: empty or minimal input, a single item, the maximum length, boundary values, negatives where allowed, duplicate or equal values, and the input of each known ghost.
- `AU-7` There are at least two known ghosts. Each is a realistic beginner mistake, and each `got` value is hand-traced in my session notes.
- `AU-8` The hints are four rungs in order: a nudge, an approach, pseudo-code of three lines at most, and a near-solution step.
- `AU-9` No rung quotes a sample value, a hidden value or a ghost `got` value.
- `AU-10` The `complexity_note` gives the time and the extra space of the reference, in 240 characters or fewer.
- `AU-11` I checked each reviewer item (`RV-*` in `reviewer.md`) that this pack should meet, and I fixed or noted every gap.
