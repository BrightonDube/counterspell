# CI/CD

Everything runs on GitHub-hosted runners, which are free for public repositories
([billing docs](https://docs.github.com/en/billing/concepts/product-billing/github-actions)).
No self-hosted runner touches this repo or the server.

| Workflow | Runs on | What it does | Gate |
|---|---|---|---|
| `ci.yml` | PRs, pushes to `brighton/kiro-setup`, `v*` tags | Path-filtered jobs: Rust fmt + clippy `-D warnings` + tests (Postgres 17 and Valkey 8 service containers, fails if zero tests ran); coverage with `cargo-llvm-cov` (summary in the run, lcov artifact, `COVERAGE_MIN_LINES` variable as the floor); web `svelte-check`, lint, Vitest, build; Docker build of all three targets (no push on PRs) | **CI OK** (required) |
| `publish-images.yml` | Called by `ci.yml` after **CI OK** on default-branch pushes and tags | Builds and pushes `counterspell-server`, `-judge`, `-migrate` to GHCR; BuildKit provenance and SBOM; cosign keyless signature; GitHub artifact attestations for provenance and SBOM. Tags also get a GitHub release with SPDX SBOMs | only after CI passes |
| `security.yml` | Every push (any branch), PRs, weekly, manual | Gitleaks and TruffleHog over full history; forbidden-file check; CodeQL (JS/TS, Rust, Actions); Semgrep CE; cargo-deny advisories/bans/sources; `npm audit`; OSV-Scanner (report to code scanning); dependency review on PRs; actionlint and zizmor | **Security OK** (required) |
| `scorecard.yml` | Default branch, weekly, ruleset changes | OpenSSF Scorecard to code scanning and the public badge | informational |
| `terraform.yml` | Changes under `infra/` | `terraform fmt`, `init -backend=false`, `validate`, TFLint. No plan or apply from CI | no-op until `infra/` exists |
| `deploy.yml` | Manual | Verifies cosign signatures, pins image digests, then SSHes with a pinned host key to a forced-command `deploy` user (`deploy/vps/`) | **disabled** until the `production` environment, its secrets and `DEPLOY_ENABLED=true` exist |

## Hardening choices (and where they come from)

- **Least privilege:** `permissions: {}` at the top of every workflow and only what each job needs
  ([GITHUB_TOKEN permissions](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/controlling-permissions-for-github_token)).
  Checkouts use `persist-credentials: false`.
- **Pinned actions:** every third-party action is pinned to a full commit SHA with the version in a comment;
  Dependabot keeps them current ([secure use reference](https://docs.github.com/en/actions/reference/security/secure-use)).
- **No script injection:** event data reaches shell only through `env:`, never `${{ }}` inside `run:`
  (same doc). zizmor and actionlint enforce this on every push.
- **Cache poisoning:** only default-branch pushes write the Cargo cache; PRs only read it.
- **Supply chain:** keyless signing and artifact attestations
  ([artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds),
  [Sigstore cosign](https://docs.sigstore.dev/cosign/signing/signing_with_containers/)).
- **Required checks with path filters:** jobs skip by path, but the single aggregate jobs **CI OK** and
  **Security OK** always report, so required checks never hang as "Expected"
  ([troubleshooting required checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/troubleshooting-required-status-checks)).
- **Deploy:** environment secrets, required reviewer (free on public repos), pinned `known_hosts`,
  forced-command key ([environments](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)).

## Repository settings

- Secret scanning and push protection, Dependabot alerts and security updates, private vulnerability reporting.
- Workflow token defaults to read-only; Actions may not approve PRs.
- Ruleset **kiro-setup protection** on `brighton/kiro-setup`: requires **CI OK** and **Security OK**, blocks force-push
  and deletion. The repository admin role is on the bypass list (mode "always") so the post-spec pusher, which
  pushes as you after the full test suite passes, can still push directly
  ([rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)).
  Everyone else, including Dependabot, must go through a PR with green checks.

## Verifying a published image

```bash
gh attestation verify oci://ghcr.io/brightondube/counterspell-server:sha-<commit> -R BrightonDube/counterspell
cosign verify ghcr.io/brightondube/counterspell-server:sha-<commit> \
  --certificate-identity-regexp '^https://github.com/BrightonDube/counterspell/\.github/workflows/publish-images\.yml@refs/(heads/brighton/kiro-setup|tags/v.+)$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```
