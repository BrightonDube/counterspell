# Lantern Ledger review

Round 1 reviewed version 1 at content hash prefix de1e3d876922.

## What the problem asks

The learner must count the changes that leave the running total at the target. The statement gives the input bounds, return type and a worked example.

## Why it tests real skill

The problem tests checking each updated balance and counting repeated visits.

## What the ghost shows

The two incorrect programs illustrate counting the starting balance and stopping at the first visit. The judge found failing inputs within its search budget, and found no counterexample to the reference control.

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
