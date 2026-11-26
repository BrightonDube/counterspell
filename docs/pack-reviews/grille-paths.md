# Review summary: grille-paths

Round 1, content hash ba3f9a46d3121d18. Written for a non-expert reader. One page.

## What the problem asks
A grille of open and bricked cells reaches the keeper; the task is the number of right-or-down routes from the top-left cell to the bottom-right.

## Why it tests real skill
The learner must sum routes cell by cell and return zero on bricks and off-grid moves; extra moves and ignored bricks both overcount.

## What the ghost shows
One ghost adds a diagonal move the grille does not allow; one treats every brick as open; one walks one row past the grille's edge.

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
