# Review summary: shared-sigils

Round 1, content hash c2a084a2842f. Written for a non-expert reader. One page.

## What the problem asks
Two grimoires each keep a column of sigil marks, smallest first. Return the marks both carry, in order, each listed once.

## Why it tests real skill
The learner must advance only the smaller head and record each shared mark once. Stepping both fingers every turn lets matches slip past, and recording every equal meeting lists one mark many times.

## What the ghost shows
One ghost steps both fingers every turn and never lets a smaller head catch up; another records every equal meeting, repeating shared marks. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that two empty columns return an empty list and that a repeated mark shares only once.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the dedupe rule and the empty-column edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, head-advancement plus dedupe lifts it to medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
