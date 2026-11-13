# Third-party notices

Counterspell is distributed under the MIT License in `LICENSE`.

The engine currently uses the following Rust crates. Their license metadata
was checked with `cargo deny check licenses` on 12 November 2026.

## Runtime dependencies

- **serde 1.0.229** — David Tolnay and contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/serde>
- **serde_json 1.0.154** — David Tolnay and contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/serde_json>

The runtime dependency tree also includes `serde_core 1.0.229`,
`serde_derive 1.0.229`, `proc-macro2 1.0.107`, `quote 1.0.47`, `syn 3.0.10`,
`unicode-ident 1.0.27`, `itoa 1.0.19`, `memchr 2.8.3`, and `zmij 1.0.23`.
The test dependency tree also includes `autocfg 1.5.0` and `libc 0.2.191`.
These crates retain their upstream MIT, Apache-2.0, or MIT/Apache-2.0
licensing as recorded in `Cargo.lock` and the crates.io package metadata.

- **uuid 1.29.2** — The uuid contributors — Apache-2.0 OR MIT —
  <https://crates.io/crates/uuid> (run-lifecycle T01, shared run ids)

## Development dependencies

- **proptest 1.12.0** — The proptest contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/proptest>

The reduced-feature proptest tree includes `bitflags`, `chacha20`, `cfg-if`,
`cpufeatures`, `rand_core`, `core_detect`, `num-traits`, `rand`, `getrandom`,
`rand_xorshift`, `regex-syntax`, and `unarray`. Their upstream license
metadata is checked by `cargo deny`; they are test-only dependencies and are
not linked into the engine's runtime dependency graph.

## Grimoire dependencies

The pack validator (`counterspell-grimoire`) adds these direct and tree
packages. `cargo deny check licenses` passed on them on 13 November 2026.

- **toml 0.8.23** — Ed Page and contributors — MIT OR Apache-2.0 — <https://crates.io/crates/toml>

The `toml` tree adds `toml_datetime 0.6.11`, `toml_edit 0.22.27`,
`toml_write 0.1.2`, `serde_spanned 0.6.9`, `winnow 0.7.15`,
`indexmap 2.14.2`, `hashbrown 0.17.1` and `equivalent 1.0.3`. Their
upstream licences are recorded in `Cargo.lock` and the crates.io metadata.

## Notice policy

This file records direct engine dependencies and the dependency-tree packages
introduced by the engine manifest. Generated or upstream license text is not
replaced here; the corresponding package metadata and lockfile remain the
source for exact copyright and notice text.
