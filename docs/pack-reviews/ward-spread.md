# Review summary: ward-spread

Round 1, content hash 73a93f8557e72fd0. Written for a non-expert reader. One page.

## What the problem asks
A row of cells and a guard count reach the keeper; the task is the number of layouts where no two guards stand on neighbouring cells.

## Why it tests real skill
The learner must run a place-or-skip walk where placing a guard forces the next cell empty; adjacency-only checks let touching guards through.

## What the ghost shows
One ghost counts every choose-k layout and ignores the spread rule; one counts pairs as spread when they only need one cell between; one indexes a row past its end.

## What could still be wrong
No unresolved pack issue was raised. The statement names the call, the bounds and every edge answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the bound keeps the search inside the declared difficulty.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
