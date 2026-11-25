# Review summary: merge-glow

Round 1, content hash 58fde84ee25a57cc. Written for a non-expert reader. One page.

## What the problem asks
Two lantern chains each hold marks sorted along the walk; the keeper must merge every mark into one ascending list.

## Why it tests real skill
The learner must walk both chains at once, always emitting the smaller mark, then drain the rest. Concatenating the chains, sorting the raw arrays, or keeping one chain each misses the merge.

## What the ghost shows
One ghost concatenates the chains; one sorts the raw arrays including strays; one returns only the first chain.

## What could still be wrong
No unresolved pack issue was raised. Equal marks keep both and stray cells are ignored.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a two-pointer merge over two walks fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
