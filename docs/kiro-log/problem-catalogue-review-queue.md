# problem-catalogue review queue (SUPERVISED tasks)

Brighton reviews each diff before the task is treated as approved. Each entry lists the task, the commit and what to check.

| Task | Commit | What to check |
|---|---|---|
| C01 | (see git log "problem-catalogue C01") | Table columns and CHECKs match design.md "Data model". `hidden_tests` has `source` and `ordinal`. `problem_versions` has no `generator_seed`. |
| C02 | (see git log "problem-catalogue C02") | Trigger rules match design.md: version update/delete refused except the one-time sandbox stamp; child insert refused after the stamp; publish gate checks 2–5 examples, four hints, a concept tag. The "unknown track" case is refused by the foreign key, not by the publish gate. |
| C03 | (see git log "problem-catalogue C03") | Submissions now need a real problem row (`submissions_problem_fk`, NOT VALID). `pg_model_interleavings` in `crates/judge/tests/model.rs` gained a setup-only `two-sum` catalogue row, marked `test-change: CAT-F1`. No assertion changed. Check that the setup change is acceptable. |
| C04 | (see git log "problem-catalogue C04") | Grant matrix: server reads public catalogue tables only; judge reads run inputs and hidden tables only; neither role writes, and neither reads `catalogue_events`. The dev cluster has its own login role named `counterspell`; the test ignores it on purpose. |
| C05 | (see git log "problem-catalogue C05") | Test kit in `crates/judge/src/catalogue/testkit.rs` repeats the helper SQL in `crates/judge/tests/catalogue_support/mod.rs`. Merge the two in a later cleanup. Canary strings are fixed constants. |
| C06 | (see git log "problem-catalogue C06") | Manifest hash: check the prefix rule (`public/`, `private/`), that `approval.toml` is excluded, and that `public_map` never holds a private path. The expected digest in `catalogue_manifest.rs` was computed with `sha256sum`. |
| C08 | (see git log "problem-catalogue C08") | Discovery filter: `_` names are never imported, drafts are skipped with a reason, and an explicit `_` slug is refused. |
| C08a | (see git log "problem-catalogue C08a and C11 to C12") | Private-tree preflight: the caller must resolve symlinks before it sets `inside_public`. Codes are pinned by tests. |
| C11 | (see git log "problem-catalogue C08a and C11 to C12") | Import planner: a same-version, different-hash import is refused with the changed files listed. Check that the planner looks only at the latest version. |
| C11a | (see git log "problem-catalogue C08a and C11 to C12") | Approval and retired-value checks. The canonical-JSON key-order property holds trivially because `serde_json::Map` is sorted by default; the explicit sort in code is still what makes the hash stable. Check the retired-value message shows only 12 characters. |
| C13 | (see git log "problem-catalogue C13") | Report model. The canary test checks the report's shape, not a planted secret: `PackReport` has no field that would carry hidden data. The stronger check belongs in C14, where real import output is rendered. |
