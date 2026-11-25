# Review summary: min-add

Round 1, content hash 8e8be105bf53. Written for a non-expert reader. One page.

## What the problem asks
A fragile inscription of round runes can only be repaired by inserting runes; return the fewest additions that make it sound.

## Why it tests real skill
The learner must track two debts that can never cancel: stray closers needing earlier opens, and open layers needing later closes. Taking a difference, a maximum, or only one debt each undercounts.

## What the ghost shows
One ghost balances the totals and treats a stray closer as cancelling an open layer it cannot reach; one forgets stranded closers entirely; one answers the larger debt instead of the sum.

## What could still be wrong
No unresolved pack issue was raised. The statement forbids removing or moving runes, so only insertions count.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the two-debt scan is one idea past a plain walk, fitting medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
