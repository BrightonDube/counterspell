# The Diagonal Total review

Round 1 reviewed version 6 at content hash prefix 1b825a0be5cc.

## What the problem asks

The learner must add the signed cells on the main diagonal of a rectangular grid. The statement gives the input bounds, return type and a worked example.

## Why it tests real skill

The problem tests bounding an index by the shorter dimension.

## What the ghost shows

The two incorrect programs illustrate stopping early at a neutral cell and reading a column instead of the diagonal. The judge found failing inputs within its search budget, and found no counterexample to the reference control.

## What could still be wrong

The independent review raised no unresolved pack issue. The bounded ghost search confirms the demonstrated mistakes within its search budget.

## The scores in words

- Correctness was strong. Samples and private expectations agreed with the sandbox.
- Clarity was strong. The input, output and worked example matched the schema.
- Originality was strong. The provenance and marker checks passed.
- Difficulty was strong. The task fit its introductory category.
- Hints were strong. The four rungs progressed without leaking case values.
- Ghost coverage was strong. Both mistakes reproduced and were independently discovered.
- Accessibility was strong. The statement works as plain text.

## Decisions for Brighton

Approved under Brighton's delegated publication authorization on 21 November 2026. An independent reviewer checked the pack. The operator ran static and rootless sandbox checks before recording the approval.
