# Release and rollback

How a release reaches the host, and how it goes back. The deploy script is `counterspell-deploy`, reached only through the forced-command entry point `counterspell-deploy-entry` (see `README.md`). Operational history goes in `OPERATIONS-LOG.md`.

## Pre-release (before the tag or merge)

1. The publish workflow has signed and attested the three images for the full commit SHA (`.github/workflows/publish-images.yml`). The deploy workflow verifies the signatures before it opens SSH (R7.1).
2. The release checklist (`.kiro/steering/release-checklist.md`) is read through for this release. Failed items are recorded with the decision to fix, defer or cut.
3. Every new migration follows the expand-only rule below. The reviewer ticks the checklist in this file.
4. Compose or Caddy changes: `bash deploy/vps/tests/caddy_shape.sh` passes, and once T14 adds it, `bash deploy/vps/tests/compose_shape.sh` passes too. Then `deploy/vps/compose.prod.yml` digests are current.
5. After a Compose or Caddy file changes on the repository side, the app role has been run on the host, so `compose.sha256` matches (`counterspell-deploy` refuses a mismatch).

## Deploy

1. Start the deploy from `main` (or a `v*` tag) in the GitHub `production` environment. Brighton approves the run (R7.1).
2. The workflow sends `deploy <sha> <server digest> <judge digest> <migrate digest>` through the forced-command key. Digests only; no tags (R7.2).
3. On the host, the script takes the lock, checks the compose files, pulls the three images by digest, runs judge preflight, runs the `migrate` container to completion, starts the stack, and checks readiness through Caddy and public health over HTTPS (R7.3 to R7.6).
4. The release record `deploy.env` becomes the current release only after both checks pass. The old current record moves to `deploy.env.previous` (R7.8).

Check the result from a laptop with the same key:

```
ssh deploy@<host> status
```

It prints the current SHA and the three digests only.

## Post-release

1. Public health answers (`https://<site>/api/health`, the UptimeRobot keyword check goes green).
2. The ops report timer ran in the last 5 minutes (`MONITORING.md`).
3. Sign in with the demo account, open one problem, run Run and Submit on a passing sample. The release is accepted only when the run finishes.
4. Record the deploy in `OPERATIONS-LOG.md` with the SHA and the outcome. No secret values.

## Migrations: expand, then contract (R7.10)

A release must work with the database schema of the release before it, because a rollback does not reverse migrations.

- **Expand** (allowed in a release): add tables, add nullable columns, add indexes, add new grants.
- **Contract** (a later release, after no running digest reads the old shape): drop or rename a column or table, change a type, tighten a constraint.
- A release never drops, renames or changes the type of anything the previous release reads.

Review checklist for each migration file:

- [ ] Does the previous release still run against the schema after this migration? Read the old code's queries for every touched table.
- [ ] No `DROP`, `RENAME`, `ALTER ... TYPE` or `SET NOT NULL` on existing data in this release.
- [ ] New columns are nullable or have a default.
- [ ] The migration runs on a copy of the database before the release (a manual dry run, recorded in `OPERATIONS-LOG.md`).

## Rollback

Use rollback when the new release fails after deploy, or misbehaves in a way a fix cannot land quickly:

```
ssh deploy@<host> rollback
```

It swaps the two release records, starts the previous digests, runs the same readiness and public checks, and never runs `migrate`. A second rollback returns to the release it replaced. If the checks fail, the release records do not change, and the stack is left as the script reports. Investigate that before doing anything else.

Automatic rollback applies only inside a deploy: when readiness or public health fails after the stack changed, the script restores the previous release itself.

## Freeze

After the feature freeze (2 Dec 2026), only security and data-safety fixes ship. Each one gets a release record in `OPERATIONS-LOG.md` (R11.6).
