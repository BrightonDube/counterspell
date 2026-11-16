---
name: pack-review
description: Use when scoring one finished Counterspell Grimoire pack in a review round, against the reviewer checklist, from the assembled folder and the stage reports, without any write access.
---

# Pack review

You score one pack in one review round. You are a separate reviewer. You have read access only, and you cannot run code.

## Procedure

1. Read the checklist at `.kiro/pipeline/checklists/reviewer.md`. Each category has items with IDs such as `RV-clarity-2`.
2. Read the public files (`problem.md`, `meta.toml`, `hints.toml`, `originality.md`) and the private files you are given in this session.
3. Read the S9 validator report and the S10 and S11 sandbox and ghost reports. They are facts. Do not re-run anything.
4. For each of the seven categories, check its items. Give a score from 1 to 5. A score of 5 means every item holds. Lower the score for each item that fails, and say which one.
5. For each score, cite evidence as a file and a pointer, such as `hints.toml rung 3` or `hidden.toml /case/3`.
6. List each problem as a finding, with an ID (`F<round>-<n>`), a severity (`blocker`, `major` or `minor`), a category, a pointer and a short message.

## Rules

- Never quote a hidden input or a hidden expected value, even to explain a finding. Use the pointer form (`hidden.toml /case/3`).
- Never quote a sample or a ghost `got` value that is hidden in the private tree.
- Do not guess what the author meant. Judge only the text and the reports you were given.
- Do not state whether the round passes. The orchestrator applies the pass rule (PIP-23) to your scores.
- Do not suggest code or a full solution.

## Output

One JSON object with these keys, followed by the findings list:

```json
{
  "correctness": 1,
  "clarity": 1,
  "originality": 1,
  "difficulty": 1,
  "hints": 1,
  "ghost": 1,
  "access": 1
}
```

Each key takes an integer from 1 to 5. Put the evidence for each score in the findings list or in a short note beside it, using pointers only.
