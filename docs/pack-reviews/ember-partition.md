# Review summary: ember-partition

Round 1, content hash 3adcc0f3595ade0d. Written for a non-expert reader. One page.

## What the problem asks
A pile of marks and a heap count reach the keeper; the task is the number of splits into that many non-empty heaps of equal total, with heaps unlabelled.

## Why it tests real skill
The learner must assign marks to heap totals and break heap-order symmetry with a fill-order rule; labelled assignments and first-fit greed both fail.

## What the ghost shows
One ghost counts labelled heaps and reports each split once per ordering; one commits marks to the first heap with room and never backtracks; one reads the first mark of an empty pile.

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
