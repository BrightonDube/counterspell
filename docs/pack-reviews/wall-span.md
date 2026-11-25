# Review summary: wall-span

Round 1, content hash ca63a6c9d912. Written for a non-expert reader. One page.

## What the problem asks
Shelf blocks at marked heights stand shoulder to shoulder; a canvas may stretch across any run whose blocks all stand at least its height. Return the largest area the wall can hold.

## Why it tests real skill
The learner must resolve each block's widest span in one pass. A global minimum times width, the tallest block alone, and adjacent pairs each miss the real shapes.

## What the ghost shows
One ghost stretches every canvas at the wall's lowest block; one only tries canvases as tall as the single tallest block; one never tries a run wider than two blocks.

## What could still be wrong
No unresolved pack issue was raised. The statement defines area as height times spanned blocks.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the resolve-on-drop stack pass is a hard step.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
