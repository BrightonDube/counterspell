# Review summary: calm-spell

Round 1, content hash a07da3bae73d. Written for a non-expert reader. One page.

## What the problem asks
A chant of a, b and # runes is split into spells by every crack; return the length of the longest stretch with no # inside.

## Why it tests real skill
The learner must reset the live length at each crack and keep the best seen. Counting all non-crack runes, keeping only the first spell, or only the last each break on mid-chant cracks.

## What the ghost shows
One ghost counts every non-crack rune and ignores the splits; one reports only the first spell; one reports only the last.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty chant and an all-crack chant answer of 0.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a split-and-measure walk fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
