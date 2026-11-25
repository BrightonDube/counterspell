# Review summary: deepest-ember

Round 1, content hash e2e7dc0db70d. Written for a non-expert reader. One page.

## What the problem asks
A warden reads an inscription of round runes; opens deepen the nesting, closes lift a layer, and strays are ignored. Return the deepest the layers ever reached.

## Why it tests real skill
The learner must keep a live depth with a floor at zero. Letting strays sink the count negative, counting opens without closing, or failing on one stray each misreads the structure.

## What the ghost shows
One ghost lets a stray closer push the count below zero, so later layers look shallow; one counts every opener and never closes; one abandons the whole inscription at the first stray.

## What could still be wrong
No unresolved pack issue was raised. The statement names stray closers and unclosed layers explicitly.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one counter with a floor fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
