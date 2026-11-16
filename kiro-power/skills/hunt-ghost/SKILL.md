---
name: hunt-ghost
description: Find, inspect, explain, fix and re-verify a failing program with the Counterspell ghost-input tools. Use when a pack's incorrect or buggy program needs a counterexample, or when a learner's solution fails and the user wants the failing input explained.
---

# Hunt a ghost input

A **ghost input** is a failing input that the engine found for a program and shrank. It is a **minimized counterexample**: the smallest failing input the engine found within its budget. Never call it the smallest possible input or a global minimum.

## Steps

1. **Reproduce.** Confirm the program is in the pack under `incorrect/` and is a `.py` file. The tools refuse any other path.
2. **Find.** Call **`find_ghost_input`** with `pack_path` and `program`. It runs in the sandbox. Read the `outcome`:
   - `Found`: a minimized counterexample with its predicate (wrong answer or crash), input, expected value, got value and shrink trace.
   - `NotFound`: no failing input within the budget. A program that only times out is reported here, because the default hunt takes wrong answers and crashes.
   - An error with `sandbox_not_configured` or `sandbox_infrastructure` means no answer was produced. It is not a wrong answer.
3. **Inspect.** Read the shrink trace and the input. A shrink step is shown only when the engine recorded it.
4. **Explain.** Call **`explain_verdict`** with the input, expected value, actual value and kind (`wrong`, `crash` or `timeout`). It returns the difference and any authored known-ghost explanation. It does not call an AI model.
5. **Shrink again if needed.** Call **`shrink_input`** on a failing input to keep the same wrong-answer or crash failure while the input gets smaller. Timeout targets are not supported.
6. **Fix the program.** Change the implementation so it is correct. Do not edit the reference, the expected values, or the known ghost to hide the failure.
7. **Validate.** Call **`validate_pack`**. It checks the pack statically and must pass.
8. **Re-hunt.** Call **`find_ghost_input`** again on the fixed program. Report `NotFound` as "no counterexample found within the budget", and say that it is not a proof that the program is correct.

## Rules

- Code-running tools (`find_ghost_input`, `shrink_input`, `run_reference`) run only in the separate rootless sandbox. Ask the user for approval before each one, and never auto-approve them.
- Never run pack code directly on the host, through a shell, Python or Node.
- Quote the outcome's own wording. Do not describe a ghost as optimal, global, or proven.
