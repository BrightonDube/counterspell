---
inclusion: fileMatch
fileMatchPattern: "crates/engine/**"
---
# Ghost engine rules

Keep the engine deterministic and separate from code execution. Use a seeded RNG and derive or record the seed with the problem, code, language, and engine version. The judge supplies the Oracle through the sandbox.

- A shrink step is accepted only when it reproduces the same failure predicate and strictly reduces the deterministic size metric.
- Record accepted shrink steps and budgets for replay and verification.
- Call results **minimized counterexamples**. Never call them “smallest possible” or claim global minimality.
- Use `proptest` for generator validity, reproducibility, shrink invariants, reproduction, and budget termination.

Source:
#[[file:docs/system-design.md]]
#[[file:docs/requirements.md]]
