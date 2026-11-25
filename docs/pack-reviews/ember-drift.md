# Review summary: ember-drift

Round 1, content hash ef0913b6a229. Written for a non-expert reader. One page.

## What the problem asks
A shelf of ember marks carries `0` where an ember went dark. Rearrange the marks so every dark ember drifts to the end while the lit embers keep their order, and return the rearranged list.

## Why it tests real skill
The learner must partition without losing the lit embers' order. Swapping dark marks with tail marks scrambles the order; sending dark marks to the front drifts them the wrong way.

## What the ghost shows
One ghost swaps each dark mark with one from the far end, scrambling lit order; another sends the dark marks to the front instead of the back. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement says in words that an empty shelf returns an empty list.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the empty-shelf edge are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, a write-cursor pass with a single idea fits easy.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, both mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.

---

Pointers and counts only. This summary must not contain a pointer into the private tree, a hidden value, a canary or a private file name. The scan `SUM-01` checks it before the pull request is opened.
