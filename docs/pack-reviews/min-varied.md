# Review summary: min-varied

Round 1, content hash 6110995ca492. Written for a non-expert reader. One page.

## What the problem asks
A frieze carved with a, b, c and d hides fragments; a fragment is varied when it holds at least one a, one b and one c. Return the length of the shortest varied fragment, or 0 when none exists.

## Why it tests real skill
The learner must shrink a window while all three runes stay inside. Checking only length-three fragments, answering the distinct-rune count, or taking the longest window each miss the true minimum.

## What the ghost shows
One ghost checks only three-rune fragments and misses longer minima; one reports the number of distinct runes; one answers the longest varied fragment instead.

## What could still be wrong
No unresolved pack issue was raised. The statement names the 0 answer when no varied fragment exists.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the shrink-while-valid two-pointer walk fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
