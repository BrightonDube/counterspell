# Review summary: rune-lineup

Round 1, content hash 1f12d7d5ac9d6ec1. Written for a non-expert reader. One page.

## What the problem asks
A row of marked candles reaches the keeper; the task is the number of orders where no two neighbours share a mark, counting equal-marked candles as distinct.

## Why it tests real skill
The learner must place candles one at a time behind a used-ledger and reject equal neighbours; factorial guesses and set collapses both fail on repeats.

## What the ghost shows
One ghost returns the factorial of the row length and ignores the neighbour rule; one collapses repeated marks into a set before arranging; one reads the first mark of an empty row.

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
