# Rust lesson: repo-skeleton T03

The workspace manifest gives every crate the same Rust edition, version, and lint rules.
A workspace keeps related packages buildable together while preserving their boundaries.
The six package names are explicit, so Cargo cannot silently add an alternate architecture.
The initial source files are intentionally tiny so later tasks can add one responsibility at a time.
# Rust lesson: repo-skeleton T04

Rust libraries can expose small constants and types without owning a process or runtime.
A crate boundary is useful documentation: `common` shares data, while `engine` stays pure.
Keeping dependencies out of these manifests prevents future sandbox concerns from leaking into them.
Doc comments become generated API documentation and explain the ownership boundary to reviewers.
