# problem-catalogue blockers and approvals

## C00 approval — 15 Nov 2026 10:26 (Brighton)

Brighton approved this spec for implementation. Details are also in `.kiro/steering/brighton-decisions.md`.

Decided now:

- The pack importer uses its own limited database role, not the database owner (security default). This overrules D4 / open question 5 (owner login, no new role).
- Open question 22, on the role only: the approved v2 hardening brief's `counterspell_importer` role wins. Table names stay as in this spec (`hidden_tests`, `known_ghosts`) unless a later task must reconcile them.

Everything else listed under C00 that Brighton did not name keeps the assumed choice in `design.md`. Stop only for a serious security, licensing-risk, cost or production-data decision that is not already settled.

## C00a — still waiting on Brighton

C00a stays unchecked. Brighton has not yet created `BrightonDube/counterspell-packs-private` or its read-only credential. No agent may create the repo, a key or a token. Tasks that need that repo should be left unchecked, recorded here, and skipped until he has done C00a.
