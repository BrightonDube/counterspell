# Review summary: candle-frequency

Round 1, content hash 4396fff0f3e7. Written for a non-expert reader. One page.

## What the problem asks
The vault ledger lists the burn size of every candle inspected. Given the sizes in order, return the size that was logged most often; a tie goes to the smallest size, and an empty ledger scores zero.

## Why it tests real skill
The learner must tally each size, then pick the largest tally with a deterministic tie rule. Answering the first size seen or the tally itself both miss the actual question.

## What the ghost shows
One ghost answers the size seen first rather than the size logged most; another answers the winning tally instead of the size. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. One review fix reworded a ghost explanation that tripped the minimality-word ban.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the tie rule and the empty-ledger edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, tally-then-scan fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
