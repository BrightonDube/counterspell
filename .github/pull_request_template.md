## What and why

<!-- One or two sentences. Link the spec: .kiro/specs/<slug>/ and task ids (e.g. T07-T09). -->

## How it was tested

- [ ] Failing test written first, then the code (TDD)
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace`
- [ ] `cd web && npm run check && npm run build` (if web/ changed)

## Safety

- [ ] No secrets, `.env` files, keys or tokens (pre-push gitleaks hook ran)
- [ ] No new dependency with a high/critical advisory or a non-permissive licence
- [ ] Untrusted code paths (judge, sandbox, MCP) reviewed for escapes, if touched
- [ ] Workflow changes keep least-privilege `permissions:` and SHA-pinned actions

## Notes for the reviewer

<!-- Open questions, follow-ups, screenshots. -->
