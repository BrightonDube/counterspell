# Review summary: kth-ember

Round 1, content hash 70b6ba2dbd7183ad. Written for a non-expert reader. One page.

## What the problem asks
A pile of marks and a one-based rank reach the keeper; the task is the mark at that rank among the low end.

## Why it tests real skill
The learner must order the pile, keep repeats as their own ranks and answer -1 when the pile is too short.

## What the ghost shows
One ghost ranks from the high end; one collapses repeats before ranking; one indexes an empty pile.

## What could still be wrong
No unresolved pack issue was raised. The statement names the repeat-rank rule and the short-pile answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, order-then-pick with a short-pile guard fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
