# Review summary: count-below

Round 1, content hash 91ecab18491629. Written for a non-expert reader. One page.

## What the problem asks
A gate line holds marks in order; the keeper must count how many sit below a limit without reading every position.

## Why it tests real skill
The learner must turn the count into an upper-bound search. Skipping equal marks, averaging the line, or reading the first position unguarded each counts wrong.

## What the ghost shows
One ghost counts only marks strictly below; one answers the line's average; one reads position zero unguarded.

## What could still be wrong
No unresolved pack issue was raised. The statement names the count on equal marks.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, one upper-bound search read as a count fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
