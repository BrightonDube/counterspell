# Hostile corpus

Every `.py` file in this directory is **data**: a program the judge runs inside the
sandbox, never on the host. Nothing here is imported, executed or spawned by test code —
files are read as text and delivered into a container as the submitted source.

The corpus exists so the scenario tests under `crates/judge/tests/sbx/` can point the
executor at behaviour that must fail safely: forged protocol lines, floods, forks,
network attempts and filesystem writes. `model_corpus::hostile_files_are_data` asserts
the corpus stays data-only.

Each file is a complete, tiny Python program named for what it tries. Comments inside a
file say which scenario uses it and what the sandbox is expected to prove.
