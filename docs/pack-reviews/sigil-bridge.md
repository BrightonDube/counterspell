# Review summary: sigil-bridge

Round 1, content hash e1ae87045cf2. Written for a non-expert reader. One page.

## What the problem asks
An arc list and a node count reach the keeper; the task is how many arcs are bridges — arcs whose removal splits their isle — where doubled arcs and self-loops never count.

## Why it tests real skill
The learner must stamp nodes on first entry and compare a child's low-link against its parent's stamp, skipping the entry arc by index; skipping by parent node lets one copy of a doubled pair count.

## What the ghost shows
One ghost calls every real arc a bridge; one skips by parent node rather than entry arc so a doubled pair still counts; one reads a stamp for a node that does not exist. Each is the smallest failing input the engine found within its budget, not the smallest possible one.

## What could still be wrong
No unresolved pack issue was raised. The statement names the call, the bounds and every edge answer.

## The scores in words
- Correctness: strong, every sample and hidden result agrees in the sandbox.
- Clarity: strong, the call, the return type and the edge cases are named.
- Originality: strong, all five record sections present and marker checks pass.
- Difficulty: strong, the bound keeps the search inside the declared difficulty.
- Hints: strong, four rungs escalate without leaking values.
- Ghosts: strong, the mistakes reproduce and the reference stands.
- Access: strong, plain wording throughout.

## Decisions for Brighton
- Approve: all gates passed under the delegated publication instruction.
