# Review summary: rune-brackets

Round 1, content hash 506b512bed2e. Written for a non-expert reader. One page.

## What the problem asks
A lintel inscription of the six bracket runes must be read for sound nesting: every closing rune has to meet the opener still waiting beneath it. Return 1 when the inscription holds, 0 when it does not.

## Why it tests real skill
The learner must hold the openers in last-in, first-out order, not just count pairs. Counting opens and closes, or checking only one rune kind, passes easy inscriptions and fails on crossed ones.

## What the ghost shows
One ghost checks only the round pair and lets crossed square runes pass; another compares the totals of each kind and misses order; a third pops whatever opener is waiting without checking its kind.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty inscription as sound.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the single stack idea with a short reference fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
