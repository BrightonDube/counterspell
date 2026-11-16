# pack-authoring-pipeline blockers and approvals

## PA00 approval — 15 Nov 2026 10:26 (Brighton)

Brighton approved this spec for implementation. Details are also in `.kiro/steering/brighton-decisions.md`.

Limits that override assumed choices where they conflict:

- Paid services and any other spending stay held. Do not buy, subscribe, or call a paid API as a paid customer.
- The private pack repo is still waiting on Brighton (same as problem-catalogue C00a). Tasks that need that repo stay unchecked until he creates it.
- Other open questions 1 to 16 keep their assumed choices, except as above. Stop only for a serious security, licensing-risk, cost or production-data decision that is not already settled.

## Assumed choices made during implementation (not blockers)

- **PA04, S1 record headings.** The requirements (PIP-8) name the record's content but not five headings. The guard requires these level-two headings, matched case-insensitively, each with some text: `Idea`, `Why it is original`, `Patterns used`, `Checklist answers`, `Third-party text avoided`. The fifth heading is the assumed choice, because PIP-3 requires the author to avoid third-party text. Brighton can rename them; the constant is `REQUIRED_SECTIONS` in `crates/grimoire/src/pipeline/originality.rs`.
- **PA06, hashing.** `counterspell-grimoire` does not compute SHA-256. Adding a hashing crate is not named in PA06, so the crate stays as it is. The manifest hash comes from `sha256sum` in `scripts/pack-assemble.sh` (PA07). The "one edited byte changes the hash" check moves to the assembly script test (PA07). PA06 tests the comparison: a missing approval gives `PUB-01`, a different hash gives `APR-01`, and an approval made before an edit becomes stale.
