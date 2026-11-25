# Review summary: equal-embers

Round 1, content hash 284c36e79b1c. Written for a non-expert reader. One page.

## What the problem asks
A hearth holds ember marks of two kinds. A span is balanced when it holds as many marks of one kind as the other. Given the marks, return the length of the longest balanced span.

## Why it tests real skill
The learner must fold two symbols into one running balance and remember where each balance was first reached. Stopping a span at its first balanced moment, or overwriting a balance's earliest position, both lose the longest span.

## What the ghost shows
One ghost ends each span at the first balanced moment; another records the latest position at a balance, shortening every span measured from it. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement now says in words that an empty hearth scores zero.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the balance rule and the empty edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the first-seen-balance map fits hard.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
