# The Longest Run review

Round 1 reviewed version 7 at content hash prefix 97c7cb826d84.

## What the problem asks

The learner must find the length of the longest consecutive run of a letter. The statement gives the input bounds, return type and a worked example.

## Why it tests real skill

The problem tests one-pass tracking of the current and longest runs.

## What the ghost shows

The two incorrect programs illustrate favoring the starting letter and counting nonconsecutive occurrences. The judge found failing inputs within its search budget, and found no counterexample to the reference control.

## What could still be wrong

Empty and single-letter inputs are covered by public samples. The fixed alphabet and retirement list leave no eligible hidden input at those sizes. The sandbox exercised both cases and checked a maximum-size hidden case.

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
