---
inclusion: fileMatch
fileMatchPattern: "web/**"
---
# Frontend rules

Use Svelte 5 runes, Bits UI, Tailwind 4, Monaco, and semantic tokens from `docs/palette.md`. Dark mode is the launch default and the UI must remain calm and keyboard-accessible.

- Use the Steel Night tokens, not ad hoc colours; bright green means pass/success and red means failures/errors.
- Honour `prefers-reduced-motion` and provide text or static alternatives.
- Use GSAP only for the landing page and Familiar. Do not add ambient IDE motion.
- Keep output as plain text and never rely on colour alone for status.
- Keep fixtures clearly labelled and never present fabricated execution results as real runs.

- Apply the Impeccable skill (`.kiro/skills/impeccable`) on all UI work. Use its principles and the relevant `reference/` playbooks. Specs, Geist and JetBrains Mono, Steel Night tokens and existing design decisions win on conflict. Do not run `npx impeccable` or download its engine; use the vendored markdown only.

Sources:
#[[file:docs/palette.md]]
#[[file:docs/system-design.md]]
#[[file:.kiro/skills/impeccable/SKILL.md]]
