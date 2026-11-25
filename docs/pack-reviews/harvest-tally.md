# Review summary: harvest-tally

Round 1, content hash 73c0bd1ad926. Written for a non-expert reader. One page.

## What the problem asks
Granary rows hold sacks, one count per row. The feast pairs each sack with sacks from different rows; return how many distinct cross-row sack pairs exist. The tally can be very large.

## Why it tests real skill
The learner must count each unordered pair of different rows once, ideally in a single pass with a running remainder. Pairing every position with every position, or counting both directions, both inflate the tally — and a small counter overflows.

## What the ghost shows
One ghost counts same-row and repeated pairs; another drops only the same-row diagonal and still double-counts. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement now says in words that a single row scores zero.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the different-row rule and the one-row edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the large-count quadratic-temptation shape fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
