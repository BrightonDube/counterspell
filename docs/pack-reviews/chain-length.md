# Review summary: chain-length

Round 1, content hash c8a256cedb8ffb53. Written for a non-expert reader. One page.

## What the problem asks
A row of lantern links is chained by index pointers; the keeper must report how many links the walk from index 0 visits.

## Why it tests real skill
The learner must follow the pointers step by step and ignore stray cells the walk never reaches. Counting the whole array, starting at index 1, or counting the -1 marks each miss the contract.

## What the ghost shows
One ghost counts every cell including strays; one starts the walk at index 1; one counts the -1 marks instead of links.

## What could still be wrong
No unresolved pack issue was raised. The statement names the walk start, the -1 ending and the stray-cell rule.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one bounded walk over an index chain fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
