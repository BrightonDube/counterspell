---
inclusion: fileMatch
fileMatchPattern: "packs/**"
---
# Grimoire authoring

Problem packs live as reviewed files under `packs/<slug>/`. Follow the pack format for the statement, metadata, input schema, generator, Python reference solution, tests, comparator, authored hints, and optional known-ghost explanations.

- Validate the reference against its tests and validate generated inputs against the schema.
- Keep the Python reference authoritative for expected outputs across launch languages.
- Write original problems and original wording only. Do not copy HackerRank, LeetCode, or other protected problem text or trade dress.
- Keep constraints, complexity declarations, and hints honest and reviewable.

Sources:
#[[file:docs/system-design.md]]
#[[file:docs/requirements.md]]
#[[file:docs/licence-rules.md]]
