# D.4 Spec 2: `ghost-engine` (Feature, Requirements-First, PBT on)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

*Rewritten 12 Nov from critique §5.5, with Brighton's decisions.*
```
Create a Requirements-First feature spec named ghost-engine. Context: #[[file:docs/system-design.md]] §8 Ghost engine and §5 ghost hunt protocol, #[[file:docs/requirements.md]] GHO-1, GHO-5, GHO-6, and #[[file:docs/product-brief.md]].
Goal: discover, reproduce and minimize counterexamples for incorrect solutions. Build a small Rust library (crates/engine, package counterspell-engine) with three clearly separated parts: input generation, failure classification, and shrinking. The pure shrinking algorithm must be separate from code execution: the engine talks to programs only through an Oracle trait that the judge implements with the sandbox. Nothing in this crate may execute code itself.
Initial supported inputs: bounded integers, strings, integer arrays, 2-D integer arrays and tuples of these (the §8 schema). No graphs, trees or arbitrary executable custom checkers in this spec.
Correctness requirements:
1. Every generated candidate satisfies the problem's input constraints.
2. A failure is independently verified against the trusted Python reference by the judge's comparator.
3. During shrinking, a candidate is kept only if it still reproduces the same failure predicate (verdict kind + exception class).
4. Define an explicit deterministic size metric (e.g. element count, then sum of absolute values, then lexicographic) and accept only steps that strictly reduce it.
5. Edge-biased deterministic generation; record seed, generator version, runtime image digest and problem version with every result.
6. Support wrong-answer and runtime-error counterexamples first; handle timeouts separately.
7. Stop when the candidate, evaluation or wall-time budget is reached (total ghost hunt ≤ 8 s).
8. Return a "minimized counterexample". Never claim global minimality.
9. Return a clear "not found / budget exhausted" status when no counterexample is found.
10. Record every accepted shrink step in order (a capped shrink trace) for the UI's shrink animation, and return the data the Bug Autopsy card needs (input, expected, actual, kind, concept tags, matching known-ghost explanation if any).
Property-based tests (enable Correctness/PBT in design, using proptest) must verify: generated candidates satisfy their validators; every returned ghost reproduces its claimed failure; every accepted shrink step reduces the size metric; equal seeds and equal deterministic oracle behaviour give equal results; the search terminates within its budgets.
Create three test problem packs, each with a correct Python reference and at least two deliberately incorrect implementations (one of them is reused later for the Kiro dogfooding demo).
Out of scope: AI hints, UI animations, complexity profiling, more language drivers.
Definition of done: three incorrect programs produce valid, reproducible minimized counterexamples within budget, and every result passes independent re-verification.
I'm new to Rust: keep the API small and explain the design trade-offs in plain English in design.md.
```
Then click **Analyze Requirements** before approving.
