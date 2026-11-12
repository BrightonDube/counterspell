# Security Policy

## Reporting a vulnerability

Please **do not open a public issue** for security problems.
Report privately via GitHub: **Security → Report a vulnerability** on this repository
(private vulnerability reporting). Include steps to reproduce and the impact.
You'll get an acknowledgement within 72 hours.

Counterspell runs untrusted user code in sandboxes; sandbox escapes, auth bypasses,
and leaked credentials are the highest priority.

## Supported versions

Only the latest commit on the default branch is supported.

## What protects this repo

- Every push and PR runs `.github/workflows/security.yml`: Gitleaks + TruffleHog
  (full history), a forbidden-file check, CodeQL (JS/TS, Rust, Actions), Semgrep CE,
  cargo-deny advisories, `npm audit`, actionlint and zizmor.
- GitHub secret scanning with push protection, and Dependabot alerts and security updates.
- Local pre-commit / pre-push Gitleaks hooks (`scripts/install-git-hooks.sh`).

If a secret is ever committed: **rotate it first**, then remove it. Deleting the
commit does not un-leak it.
