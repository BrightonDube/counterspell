# Steering questions for Brighton

This non-interactive setup found the following documentation conflicts. The current steering request was applied where it names a file or narrower implementation rule; the system-design decision log remains authoritative for architecture and security.

## Rust steering filename
- **Conflict:** `docs/product-brief.md` (People and process), `docs/system-design.md` (§0 and §13), and `docs/kiro-playbook.md` (A.1) refer to `rust-learning.md`; the current setup request asks for `rust-teaching.md`.
- **Choice:** created `.kiro/steering/rust-teaching.md` with the same required teaching behavior. No duplicate alias was created, so the requested filename is the active rule.

## GSAP scope
- **Conflict:** `docs/product-brief.md` (Stack) names GSAP for the landing page, Familiar, and shrink animation; the current tech steering decision says “GSAP only for landing + familiar.”
- **Choice:** the tech and frontend steering files limit GSAP to the landing page and Familiar. The shrink animation remains required by `docs/system-design.md` and `docs/palette.md`, but should use simple non-GSAP transitions or a static reduced-motion presentation.
