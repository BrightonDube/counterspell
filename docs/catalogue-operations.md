# Catalogue operations runbook

How an operator imports, validates, publishes and seeds catalogue problems with
the `counterspell-catalogue` binary (`crates/catalogue-cli`, decision D2; moved
out of `crates/judge` per the 15 Nov decision). This document
names environment variables; it never contains a value, a credential or a URL.
Source of truth: `.kiro/specs/problem-catalogue/design.md`.

## Environment

| Variable | Meaning |
|---|---|
| `CATALOGUE_DATABASE_URL` | Postgres connection for the operator role. The name is logged on misuse; the value is never printed. |
| `CATALOGUE_OPERATOR` | Operator label written to `catalogue_events` (defaults to `$USER`). |

All commands share exit codes: `0` ok, `1` refused packs or failed checks,
`2` usage, infrastructure or missing-runner error.

## The private checkout (CAT-PT14)

Hidden data (hidden tests, generator seeds, the reference, known ghosts,
approval file) lives in the private repository, not in `packs/`. The operator
makes a local checkout **outside the public repo** using a read-only deploy key
or token that is handed to git by the operator — never stored in the repo,
never passed to the binary — and **deletes the checkout after the import**.
The importer itself never runs git, never fetches and never holds a credential
(CAT-PT4); it only reads the directory passed as `--private-root`.

`--private-root` must exist, be readable, and sit outside the public packs
tree. A missing or unreadable root stops the whole run before any database
connection. Its layout mirrors the public tree: `<private-root>/packs/<slug>/`
holds that slug's private files plus `approval.toml`; `retired-values.toml`
sits at the root.

## Commands

```text
counterspell-catalogue import [--slug <slug>] [--packs <dir>] --private-root <dir> \
    [--static-only | --sandbox] [--dry-run] [--deny-warnings] [--json]
counterspell-catalogue validate-sandbox <slug> [--packs <dir>] --private-root <dir>
counterspell-catalogue publish <slug> [--version <n>]
counterspell-catalogue unpublish <slug>
counterspell-catalogue list
counterspell-catalogue seed-dev --static-only --private-root <dir>
counterspell-catalogue seed-demo --private-root <dir>
```

### import

Reads the public pack tree and the private checkout, joins them by slug, runs
the static and (unless `--static-only` or `--dry-run`) sandbox checks, and
writes one immutable version per pack that passes. Per-pack lines report
`created`, `unchanged` or `refused:<code>`.

- `--dry-run` plans every pack and writes nothing — no catalogue rows and no
  taxonomy rows. Use it to preview a checkout.
- `--static-only` runs the pipeline up to (but not into) the sandbox: taxonomy
  syncs, the approval gate still applies, no `hidden_*` rows are written.
- `--deny-warnings` turns grimoire warnings into refusals.
- `--json` prints one report object per line with slug, outcome, counts and a
  hash prefix — never private paths or values.

### The draft gate and the bump rule

A pack is imported only when `meta.toml` says `status = "published"`;
`status = "draft"` reports `refused:draft_pack`. A stored version never
changes: new content requires the pack author to bump `pack_version` in
`meta.toml` — a same-version content change reports `refused:version_regressed`
(CAT-I9). `approval.toml` in the private tree must name the combined content
hash; a mismatch refuses before the sandbox runs (CAT-PT7). Directory names
starting with `_` are test fixtures and are always skipped.

### validate-sandbox

Runs the sandbox fill for one slug against the latest stored version: the
reference answers the samples, curated hidden inputs and generated inputs
inside the judge's sandbox daemon, the pack's comparator checks every answer,
and the version is stamped `sandbox_validated_at`. Requires a configured
`PackRunner`; without one the command reports `SBX-07` and exits 2. A version
that is already stamped reports `already_validated`; a changed private tree
reports `private_tree_changed`.

### publish / unpublish

`publish <slug>` pins the latest sandbox-validated version (or `--version <n>`
explicitly) and marks the problem `published`. The database trigger is the
gate: the version must be sandbox-validated and carry 2 to 5 examples, exactly
four hints and at least one concept tag — a refusal prints the trigger's own
check text. `unpublish <slug>` sets the state back to `unpublished` and stamps
`unpublished_at`; it keeps `published_version` as history and is idempotent —
a second run reports `already_unpublished` and writes no event.

### list

Prints `slug state version` for every catalogue problem (`-` when no version
is pinned), or one JSON object per line with `--json`.

### seed-dev / seed-demo

Both commands share one path. `seed-dev --static-only` and `seed-demo` scan the
pack trees, import every *eligible* pack (published status with a private
part) and skip drafts and incomplete packs silently; when nothing is eligible
they print `no eligible packs` and exit 0. `seed-demo` additionally publishes
each imported problem's latest validated version, so it needs the sandbox
runner. Both require `--private-root`. Until the starter packs are approved
and published, a seed run imports nothing — that is the intended behaviour,
not a fault.

## Retired values and rotation (C40)

`<private-root>/retired-values.toml` records `[[retired]]` hashes of hidden
values that were ever exposed (for example in public git history). At sandbox
import, generated and curated hidden values are checked against the retired
list; a slug whose hidden values collide with a retired hash needs a
`[[rotated]]` record — new generator seed and curated set — before its import
proceeds (CAT-PT8, CAT-PT13). Rotation is a human task: change the generator
seed (and any colliding curated inputs), record the rotation, and re-run
`validate-sandbox`.

## A typical dry run

```text
counterspell-catalogue import --dry-run --packs packs --private-root <checkout>
```

prints one report line per discovered pack plus one per orphan private
directory, writes nothing, and exits 0 only when every pack would be created
or unchanged.
