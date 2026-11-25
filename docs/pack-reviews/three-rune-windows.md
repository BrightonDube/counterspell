# Review summary: three-rune-windows

Round 1, content hash 5f69592002b4. Written for a non-expert reader. One page.

## What the problem asks
A mosaic band painted with a, b and c hides panes; a pane is complete when it shows all three runes at least once. Count the complete panes.

## Why it tests real skill
The learner must count, per start, how far the first complete window can extend and sum the tails. Checking only length-three panes, presence-only, or an off-by-one tail each undercount.

## What the ghost shows
One ghost counts only three-rune panes; one answers 1 when any complete pane exists; one drops the last pane off every start's tail.

## What could still be wrong
No unresolved pack issue was raised. The statement names panes as contiguous stretches of any length.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the first-cover-tail counting argument is the hardest pattern in the batch.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
