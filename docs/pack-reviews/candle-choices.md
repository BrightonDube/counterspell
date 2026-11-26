# Review summary: candle-choices

Round 1, content hash 105a98f9cbb2d926. Written for a non-expert reader. One page.

## What the problem asks
A row of candles and a pick count reach the keeper; the task is the number of ways to choose that many candles.

## Why it tests real skill
The learner must count choose-k subsets without repetition; a naive multiply overcounts and a wrong stopping rule undercounts.

## What the ghost shows
One ghost multiplies n times k and calls it the count; one divides by zero when no candles are wanted; one reads a cell a row does not have.

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
