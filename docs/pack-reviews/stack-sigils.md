# Review summary: stack-sigils

Round 1, content hash 9215d7342b1f. Written for a non-expert reader. One page.

## What the problem asks
Marks above zero are laid on a sigil stack and zeros lift the top away; report the mark left on top, or -1 when the stack stands bare.

## Why it tests real skill
The learner must simulate a real stack and guard the bare case. Lifting the bottom, counting instead of reading the top, or popping unguarded each fail on ordinary sequences.

## What the ghost shows
One ghost lifts the bottom mark like a line and keeps the wrong layer; one pops without checking and crashes on a bare lift; one reports how many marks remain rather than which sits on top.

## What could still be wrong
No unresolved pack issue was raised. The statement spells out that lifting a bare stack does nothing.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a direct simulation with one guard fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
