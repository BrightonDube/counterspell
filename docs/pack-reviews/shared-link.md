# Review summary: shared-link

Round 1, content hash 9fa610840ebc31cf. Written for a non-expert reader. One page.

## What the problem asks
Two walks share one vault of cells from different heads; the keeper must return the first index reachable from both, or -1.

## Why it tests real skill
The learner must collect one walk's indices and test the other's — lockstep positions and pointer comparisons miss meets at different depths. Reporting the b-head, stepping in lockstep, or comparing nxt[ha] to nxt[hb] each misses the contract.

## What the ghost shows
One ghost reports the b-head as the answer; one steps both walks in lockstep; one compares the two heads' pointers.

## What could still be wrong
No unresolved pack issue was raised. The two-heads rule and the -1 miss are named.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a set-intersection over two walks fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
