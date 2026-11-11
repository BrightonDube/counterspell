# D.11 Spec 9: `hints` (Quick Spec)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
Quick spec: hints. Context: #[[file:docs/system-design.md]] (hint ladder, AI hints) and #[[file:docs/licence-rules.md]]. Authored hint ladder per problem, plus an optional AI hint grounded in the ghost input via one OpenAI-compatible client: primary gpt-6-luna, fallback Groq openai/gpt-oss-20b on error/timeout; per-user daily cap; never send the full solution; label AI output as an AI hint. The same client serves the familiar's speech bubble (spec 12). Keys from env vars only.
```
