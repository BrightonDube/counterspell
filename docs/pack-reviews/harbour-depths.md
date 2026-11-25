# Review summary: harbour-depths

Round 1, content hash f2c822ca3bb8. Written for a non-expert reader. One page.

## What the problem asks
A harbour wall is a row of posts with heights; water held between two posts fills to the lower post's height across the gap. Return the most water any pair of posts can hold.

## Why it tests real skill
The learner must see that the lower post bounds the pair, so only stepping the lower end inward can improve. Moving both ends at once skips the best pairings, and measuring across the posts instead of the gaps over-counts.

## What the ghost shows
One ghost steps both ends inward each turn; another counts the posts in the span rather than the gaps between them. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that a row with fewer than two posts holds none.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the gap-versus-post measure and the small-row edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the bounding-end insight lifts it out of easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
