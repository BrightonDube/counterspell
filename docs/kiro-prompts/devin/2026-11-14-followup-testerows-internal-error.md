# Devin follow-up fix — `runOutcomeLabel` uses a code the judge never sends (14 Nov 2026)

Same defect class as the reducer fix you just landed (`1819b5d`), one file over.

`web/src/lib/workspace/test-rows.ts` `runOutcomeLabel(terminal)` ends with:

```ts
return terminal.code === 'internal_error'
  ? 'Internal error (not a wrong answer)'
  : 'Run failed';
```

`internal_error` is a `RunStatus`, never a `FailureReason` wire code — the judge
sends `retries_exhausted` for infrastructure exhaustion, so an infra-failed run
now shows "Run failed" instead of "Internal error (not a wrong answer)".

Do (strict TDD):

- RED: add/adjust tests where `runOutcomeLabel` is tested (find the existing
  test file for `test-rows.ts`; if none covers this function, add cases in the
  nearest existing test file rather than creating a new one unless needed) using
  ONLY real codes: `retries_exhausted` → the internal-error label,
  `cancelled`/`problem_unavailable`/`language_unavailable`/`expired` → the
  failure label. Do not use `code: 'internal_error'` — it is not a real payload.
- GREEN: fix the check in `test-rows.ts` to match the reducer's
  `statusForErrorCode` mapping (`expired`/`retries_exhausted` are the only
  non-`failed` outcomes; for the label, `retries_exhausted` is the one that
  means "internal error, not a wrong answer" — check `run_state.rs` and the
  label's intent before choosing).
- Verify: `cd web && npm run check && npm test && npm run build` — all clean.

Rules: stay on `brighton/kiro-setup`, never touch `main`, one commit
(`fix(web): ...`), do not edit `.kiro/specs/**` or `tasks.md`, do not weaken
tests. If the defect does not reproduce, write what you found in
`docs/kiro-log/` instead. Finish with a short summary: files changed, commit,
test names, verification results.
