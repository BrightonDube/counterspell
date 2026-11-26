# Review summary: next-rune

Round 1, content hash 19be250a3ef9322d. Written for a non-expert reader. One page.

## What the problem asks
A comma-joined dictionary reaches the keeper; the task is counting the distinct runes that can extend a prefix inside the dictionary.

## Why it tests real skill
The learner must read the trie node's fan-out: each different rune right after the prefix counts once, and an exact entry offers no rune.

## What the ghost shows
One ghost counts completions instead of distinct runes; one invents a rune for an exact entry; one reads past an empty gap.

## What could still be wrong
No unresolved pack issue was raised. The statement names the exact-entry rule, the distinct-rune count and the empty-prefix behaviour.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a scan collecting a rune set over entries fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
