# Live typing guess — grammar sources and size budget (T46)

Recorded 2026-11-22 after Brighton's go-ahead for the stretch.

## Decision: go

Tree-sitter parsing for the guess runs in a Web Worker, loaded lazily the
first time the user switches the guess on. Nothing is added to the build yet
(T47/T48 wire it).

## Sources

| Asset | npm package | Version | Licence |
|---|---|---|---|
| Parser runtime | `web-tree-sitter` | 0.27.2 | MIT |
| Python grammar | `tree-sitter-python` | 0.25.0 | MIT |
| JavaScript grammar | `tree-sitter-javascript` | 0.25.0 | MIT |

Each grammar package ships a prebuilt `tree-sitter-<lang>.wasm`, so no build
step is needed: the worker fetches the two `.wasm` files from `web/static/`
(or the package `node_modules` path resolved by Vite's `?url` import) plus the
runtime's `web-tree-sitter.wasm`.

Rejected: `tree-sitter-wasms` 0.1.13 bundles every grammar (~51 MB unpacked)
and is Unlicense; per-grammar packages are smaller and clearly MIT.

## Sizes (measured from the published tarballs)

| File | Raw | gzip |
|---|---|---|
| `web-tree-sitter.wasm` | 210,688 B | 83,723 B |
| `tree-sitter-python.wasm` | 457,883 B | 65,455 B |
| `tree-sitter-javascript.wasm` | 411,770 B | 49,687 B |
| **Total** | 1,080,341 B | **198,865 B (~199 KB)** |

## Budget

600 KB gzip total for the guess feature, set by T46. Measured gzip is ~199 KB,
about one third of the budget — the stretch proceeds within budget. The worker
script itself (`worker.ts`) is a few KB and the heuristics module is plain
TypeScript; both fit comfortably inside the remainder.

## Notes for T47/T48

- Grammar `.wasm` files load only on first enable, never at page load, and
  never leave the browser — the guess is local, labelled, and cannot reach the
  mood path or the network (R14.1, R14.6).
- If a third language grammar is ever wanted, re-check the budget before
  adding it.
