# D.10 Spec 8: `language-drivers` (Feature, Design-First)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Create a feature spec named language-drivers (Design-First). Context: #[[file:docs/system-design.md]] §6 Languages as config and §8b bench mode. Add JavaScript (node) to languages.toml with run commands and a stdin/stdout driver that matches the Python driver, plus bench mode for the Efficiency Familiar. Drivers print only raw answers; the worker compares and times. Java (eclipse-temurin, not openjdk; -Xss16m to start, test 8–16m on deep-recursion problems and keep the smallest that passes) and C++ (gcc) are tasks labelled [stretch], done only after the core is freeze-safe. The Python reference is authoritative for expected outputs in every language; there are no per-language references. Properties: for every problem, each driver round-trips any generated input and output exactly, and a per-language known-correct test fixture matches the Python reference's outputs on generated inputs.
```
