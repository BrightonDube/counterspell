# Review summary: twin-ledgers

Round 1, content hash 7c7583b5f734. Written for a non-expert reader. One page.

## What the problem asks
Two vault ledgers list rows of key and amount, and a key can appear on many rows. Total each key per ledger, then return how many keys agree across both ledgers; a key in only one ledger never counts.

## Why it tests real skill
The learner must sum repeated keys into per-ledger maps, then join on shared keys. Comparing rows by position, or overwriting a repeated key's earlier rows, both corrupt the join.

## What the ghost shows
One ghost compares ledger rows position by position; another keeps only a repeated key's last row instead of summing them. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. Review fixes added the indexing concept tag a ghost legitimately uses and aligned the stated bounds with the matrix schema.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the total rule, the one-ledger rule and the empty edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a keyed join over two maps fits medium.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
