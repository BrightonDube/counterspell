# Review summary: even-offering

Round 1, content hash 316e4189593430. Written for a non-expert reader. One page.

## What the problem asks
A row of offering bowls splits into unbroken runs for keepers; the keeper must name the smallest achievable value of the heaviest share.

## Why it tests real skill
The learner must search the cap answer space with a greedy feasibility scan. Dividing the total evenly, answering the largest bowl, or summing everything each misses the split.

## What the ghost shows
One ghost answers total-over-keepers; one answers the largest bowl; one ignores the keeper count entirely.

## What could still be wrong
No unresolved pack issue was raised. The statement keeps runs unbroken and parts within the row length.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one answer-space search with a greedy feasibility scan fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
