# Review summary: leaf-lanterns

Round 1, content hash 084a33ad2b4e7a8a. Written for a non-expert reader. One page.

## What the problem asks
A tree stored as three index arrays holds glowing nodes; the keeper must count how many reachable nodes carry no children.

## Why it tests real skill
The learner must walk the tree from index 0 and count leaves among the nodes actually reached, ignoring stray cells. Counting every cell, counting parents, or following an unchecked child index each misses the contract.

## What the ghost shows
One ghost counts every cell in the array; one counts branching nodes instead of leaf nodes; one follows a dangling child index and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement names the index-0 start, the -1 child mark and the reachable-only rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one bounded walk counting childless nodes fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
