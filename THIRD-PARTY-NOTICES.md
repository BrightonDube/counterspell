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

## Development dependencies

- **proptest 1.12.0** — The proptest contributors — MIT OR Apache-2.0 —
  <https://crates.io/crates/proptest>

The reduced-feature proptest tree includes `bitflags`, `chacha20`, `cfg-if`,
`cpufeatures`, `rand_core`, `core_detect`, `num-traits`, `rand`, `getrandom`,
`rand_xorshift`, `regex-syntax`, and `unarray`. Their upstream license
metadata is checked by `cargo deny`; they are test-only dependencies and are
not linked into the engine's runtime dependency graph.

## Notice policy

This file records direct engine dependencies and the dependency-tree packages
introduced by the engine manifest. Generated or upstream license text is not
replaced here; the corresponding package metadata and lockfile remain the
source for exact copyright and notice text.
