# Review summary: grid-nook

Round 1, content hash 156a04d96699. Written for a non-expert reader. One page.

## What the problem asks
A grille of open and bricked cells reaches the keeper; the task is the size of the nook holding the top-left cell, joined only up, down, left and right.

## Why it tests real skill
The learner must flood the grille four ways and stop at bricks and borders; a diagonal spill or a whole-grille count both fail on the first cut-off cell.

## What the ghost shows
One ghost counts every open cell, nook or not; one lets the flood leak through diagonals; one steps past the last column without a bounds check. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

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
