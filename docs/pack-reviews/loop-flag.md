# Review summary: loop-flag

Round 1, content hash 073abff8c959b615. Written for a non-expert reader. One page.

## What the problem asks
A chain may be knotted so the walk revisits a cell forever; the keeper must flag 1 for a loop, 0 when it ends.

## Why it tests real skill
The learner must remember visited indices or the walk never ends on a knot. Reporting every chain free, checking only self-loops, or checking only the first cell each misses real loops.

## What the ghost shows
One ghost reports every chain as free; one checks only self-loops; one checks only whether index 0 points to itself.

## What could still be wrong
No unresolved pack issue was raised. Unreachable loops are named as not counting.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a visited-set flag over a knotted chain fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
