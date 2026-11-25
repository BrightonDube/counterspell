# Review summary: longest-rite

Round 1, content hash b59bce13d372. Written for a non-expert reader. One page.

## What the problem asks
A tangled rite inscription of round runes hides sound passages between broken stretches; return the length of the longest sound run.

## Why it tests real skill
The learner must measure between breakage boundaries, not over the whole inscription. Judging only a prefix, all-or-nothing on the whole, or counting rune kinds each miss interior passages.

## What the ghost shows
One ghost stops at the first stray rune and ignores later passages; one demands the whole inscription be sound; one doubles the smaller rune-kind count and imagines pairs order forbids.

## What could still be wrong
No unresolved pack issue was raised. The statement names sound as ordered closing, not just counting.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the boundary-stack measurement is the hardest pattern in the batch.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
