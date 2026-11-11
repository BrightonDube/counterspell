# Counterspell repository guidance
Read `docs/` first; those documents define product, requirements, design, palette, licence, and Kiro process.
Use `.kiro/steering/` for scoped implementation rules and inclusion modes.
Follow the canonical Cargo and web layout described in `docs/system-design.md`.
Treat `crates/judge/` as the security boundary and keep untrusted code sandboxed.
Do not change fixed decisions without asking Brighton.
