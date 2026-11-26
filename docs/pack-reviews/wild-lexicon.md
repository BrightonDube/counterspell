# Review summary: wild-lexicon

Round 1, content hash 1981c30617e45f07. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is counting the distinct entries a star-marked search pattern matches.

## Why it tests real skill
The learner must walk a pattern whose wild rune takes any run of runes — including none — and retry every branch the wild rune opens.

## What the ghost shows
One ghost treats the wild rune as exactly one rune; one strips it and compares literally; one reads the first rune of an empty gap.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty-run rule, the bare-star answer and the distinct-entry count.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a two-dimensional reachability walk per entry fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
