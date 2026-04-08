# sorts

All sorting algorithm implementations, organised by family. Each subfolder is a sort family (e.g. `merge_sorts/`, `shell_sorts/`) containing the algorithm variants and their registration logic.

## How sorts are structured

Every sort implements `SortAlgo<T, U>` — generic over the element type `T` and a `SortLogger<T>` that captures every comparison, swap, and write. This dual-purpose design means the same code runs at full speed in benchmarks (with `NoOpLogger`) and produces frame-by-frame visualisation data (with `VisualizerLogger`).

## Registration

Sorts self-register at program startup — there is no central list to maintain. Two mechanisms coexist:

1. **`create_sort!` + `#[derive(SortRegistry)]`** — the older path. The macro generates a monomorphic wrapper, a `linkme` distributed-slice entry for benchmarks, and a `#[ctor]` hook for the runtime registry.
2. **`sort_family!`** — the newer declarative macro. Defines a variant tree of generic parameters and generates all combinations automatically. Used by merge sorts to register 80+ variants from one invocation.

## Migration status

Only `merge_sorts` is currently wired into the active dispatch (`mod.rs`). The remaining families (`bubble_sorts`, `shell_sorts`, `quick_sorts`, etc.) are compiled but commented out of the dispatch tree, pending migration to the `sort_family!` system.

## Files in this directory

- `mod.rs` — top-level dispatch: routes a `choice` path to the correct family's `fn_sort`.
- `annotations.rs`, `example_generic_sort.rs` — reference/template files (disconnected).
