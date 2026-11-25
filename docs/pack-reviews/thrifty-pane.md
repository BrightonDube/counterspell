# Review summary: thrifty-pane

Round 1, content hash 94a4490643bb. Written for a non-expert reader. One page.

## What the problem asks
A glazier cuts panes from a strip of a, b and c runes; a pane is thrifty when it shows at most two different runes. Return the length of the longest thrifty pane.

## Why it tests real skill
The learner must grow a window and retreat once a third rune arrives. Allowing three runes, measuring same-rune runs, or all-or-nothing on the strip each distort the answer.

## What the ghost shows
One ghost allows up to three runes and returns too-long panes; one measures only same-rune runs; one requires the whole strip to qualify or answers 0.

## What could still be wrong
No unresolved pack issue was raised. The statement names the empty strip answer of 0.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the grow-and-retreat window with a frequency count fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
