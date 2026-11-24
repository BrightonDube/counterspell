# Language Consistency Tasks

- [x] **LC01 Reproduce the public API and blank-editor mismatch**
  - `LanguageCatalog::launch` statically advertised all four languages while the
    workspace dropdown consumed the global list, so a problem with only
    python/javascript starters offered Java and C++ into a blank editor
    (`editor-session.ts` `?? ''` fallback). Reproduced against the testkit
    fixture before the fix.
- [x] **LC02 Add failing catalogue and API contract tests**
  - `seams.rs` unit tests: `from_enabled` lists only enabled drivers with their
    own `max_source_bytes`; the repo catalogue lists all four.
  - `http.rs`: `/api/languages` omits a disabled driver.
  - `post_runs.rs`/`ghost_runs.rs`: an enabled-language-but-no-starter pair →
    `400 unsupported_language`.
  - `pg_catalogue_lookup.rs`: `supports_language` reads starter rows; wrong
    version pins answer false.
  - `pg_catalogue_publish_gate.rs`: a three-starter version is refused.
  - `pg_catalogue_api.rs`: `enabled_languages` is all four on a complete pack.
  - `ProblemWorkspace.test.ts`: the picker offers only `enabledLanguages`.
- [x] **LC03 Make enabled languages depend on complete starters and drivers**
  - `languages.rs` moved to `counterspell-common` (judge re-exports it), so the
    server reads the same enabled-driver contract without crossing the sandbox
    boundary — the `the_server_cannot_reach_docker_or_the_judge` boundary test
    caught and now passes this move. `main` fails startup when `LANGUAGES_FILE`
    is missing or invalid.
- [x] **LC04 Harden workspace language selection and persistence**
  - `ProblemWorkspace.svelte` filters the picker to the problem's
    `enabledLanguages`; `EditorPanel` already falls back to `offered[0]`, so a
    stale choice cannot produce a blank editor.
- [x] **LC05 Reject unsupported server and judge requests clearly**
  - `ProblemLookup::supports_language` (Pg + in-memory) gates both
    `POST /api/runs` and `POST /api/runs/ghost` with a bounded
    `400 unsupported_language`; judge dispatch stays fail-closed through the
    shared catalogue.
- [x] **LC06 Require all four starters in the publication gate**
  - Migration `20261205120000_publish_gate_starters.sql` extends
    `catalogue_publish_gate()` with the four-launch-language starter check;
    `insert_published_fixture` and `static_import` now carry all four starters
    like real packs.
- [ ] **LC07 Run four-driver sandbox and live workspace verification**
  - Sandbox-driver proof is environment-gated (`SANDBOX_DOCKER_SOCKET`): the
    `sbx_*` and `pa27` suites run in CI's sandbox job; live workspace
    verification follows the merge deploy.
