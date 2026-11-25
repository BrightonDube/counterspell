# Review summary: candle-compact

Round 1, content hash c3630015676c. Written for a non-expert reader. One page.

## What the problem asks
A row of candles carries sigil marks and one cursed sigil must be purged. Return how many candles keep a mark other than the cursed sigil.

## Why it tests real skill
The learner must count the survivors, not the purged. Removing only the first cursed mark or reporting the count of candles removed both give wrong answers on ordinary rows.

## What the ghost shows
One ghost removes only the first cursed mark before counting; another reports the purged count, answering the question backwards. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that an empty row keeps none.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the empty-row edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a counting write-cursor idea fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
