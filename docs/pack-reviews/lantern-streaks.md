# Review summary: lantern-streaks

Round 1, content hash 90d7b07440f1. Written for a non-expert reader. One page.

## What the problem asks
A row of lanterns ends a street; each is lit or dark. Given the marks in order, return how many lanterns are lit in a row at the end of the street. The count stops at the first dark lantern from the back.

## Why it tests real skill
The learner must walk the row from the end and stop early. Summing every lit lantern or measuring the first run instead both give wrong answers on ordinary rows.

## What the ghost shows
One ghost counts every lit lantern anywhere in the row; another measures the run at the start rather than the end. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement now says in words that an empty street scores zero.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the empty-street edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a short reference with a single key idea fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
