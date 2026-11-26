# Review summary: rune-gap

Round 1, content hash ca7c4ddb6c0d40a2. Written for a non-expert reader. One page.

## What the problem asks
A pile of marks and a satchel cap reach the keeper; the task is the number of picks whose total stays at or under the cap.

## Why it tests real skill
The learner must count subsets under a running cap and prune when it is spent; counting only exact fits or only strict fits both miss.

## What the ghost shows
One ghost counts only subsets hitting the cap exactly; one counts subsets strictly under it and drops exact fits; one reads the first mark of an empty pile.

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
