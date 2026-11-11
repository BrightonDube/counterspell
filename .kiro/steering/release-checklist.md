---
inclusion: manual
---
# Release checklist

Use this on freeze day. Read sections 14 and 7 of the system design and check:

- The real fail → queue → sandbox → minimized counterexample flow works on the VPS.
- The trusted worker owns tests, comparison, deadlines, and verdicts; no untrusted code reaches the host or main Docker daemon.
- Sandbox isolation, cleanup/reaping, output limits, server-side VAL-1–VAL-5, rate limits, and ownership checks are tested and documented.
- The Postgres queue, leases, retries, terminal writes, and SSE replay work; run events reveal no hidden data.
- Accessibility and privacy checks pass: keyboard path, reduced motion, plain-text output, status icons/text, secure cookies, and no secrets in the repo.
- Load-test 20 concurrent submits with `JUDGE_CONCURRENCY=2` before raising it.
- Verify demo-account access, 10 original problems, Python/JavaScript launch support, HTTPS, backups, health checks, and the MIT/licence notices.
- Record any failed check and the decision to fix, defer, or cut before submission.

Source:
#[[file:docs/system-design.md]]
#[[file:docs/requirements.md]]
