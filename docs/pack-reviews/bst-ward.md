# Review summary: bst-ward

Round 1, content hash de01352743a78b07. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays must answer `1` when the search-order rule holds at every reachable node, `0` when it breaks.

## Why it tests real skill
The learner must carry a range down each branch — a node's mark bounds its whole subtree, not just its children. Checking only neighbours, allowing equality, or reading an unchecked index each misses violations.

## What the ghost shows
One ghost checks only parent-child pairs and misses far violations; one allows equal marks where the strict order forbids them; one reads `vals[left[i]]` unchecked and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement names the strict inequality, the empty-tree answer and the reachable-only rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a range-carrying walk fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
