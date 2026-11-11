# D.18 Plan mode: deploy

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

```
(Plan mode) Read #[[file:docs/system-design.md]] §10 Deployment and §11 Cost. Produce a step-by-step plan to deploy to an OVH VPS-2 (4 vCPU, 8 GB, 75 GB) with Docker Compose + Caddy (HTTPS) on the main daemon, a Postgres volume and off-box backups, Valkey for cache and rate limits only, a separate rootless Docker daemon for sandboxes only (unprivileged `sandbox` user, cgroup v2 delegation verified, images pre-pulled), the judge mounting only that rootless socket with no access to the main daemon, JUDGE_CONCURRENCY=2, and a GitHub Actions deploy. Don't change files; list commands I run myself.
```
