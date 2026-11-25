# The Sigil Stack

## Task
An acolyte lays numbered marks onto the sigil stack one at a time. Every mark above zero is laid on top; every `0` lifts the mark currently on top away (lifting a bare stack does nothing). Call `top_sigil(ops)` and return the mark left on top once the sequence ends — or `-1` when the stack stands bare.

## Input
A list of marks. Each is `0` or a positive number at most `99`.

## Output
Return the mark on top after every step, or `-1` when the stack is bare.

## Examples
- `top_sigil([3, 7, 0])` returns `3`. The `7` is lifted, exposing the `3`.
- `top_sigil([5])` returns `5`. One mark stays on top.
- `top_sigil([4, 0, 0])` returns `-1`. Two lifts leave the stack bare.

## Constraints
The sequence has at most 40 entries.
