# Review summary: stem-harvest

Round 1, content hash e1019612f682d0f9. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is summing the depth of every distinct word, where depth is the longest leading word's length.

## Why it tests real skill
The learner must hang each word from its deepest proper stem — not its shortest stem and not its own length.

## What the ghost shows
One ghost adds the word's own length; one hangs it from the shortest stem; one reads the first rune of an empty gap.

## What could still be wrong
No unresolved pack issue was raised. The statement names the longest-stem rule, the depth-zero answer and the distinct-word total.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a per-word deepest-stem search fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
