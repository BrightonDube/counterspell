# Review summary: undo-ritual

Round 1, content hash 37dd928c10ca. Written for a non-expert reader. One page.

## What the problem asks
A chant writes runes while each mark erases the rune still standing just before it. Return the text that survives.

## Why it tests real skill
The learner must erase the last written rune, not the first and not the next. An erase on an empty page must be guarded or it crashes.

## What the ghost shows
One ghost reads the mark as erasing the next rune; one erases the first written rune; one erases without checking the page and crashes.

## What could still be wrong
No unresolved pack issue was raised. The statement spells out that a mark with nothing written erases nothing.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the buffer is one stack, and str-return assembly lifts it to medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
