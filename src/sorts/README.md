# sorts

All sorting algorithm implementations, organised by family. Each subfolder is a sort family (e.g. `merge_sorts/`, `shell_sorts/`) containing the algorithm variants and their registration logic.

## How sorts are structured

Every sort implements `SortAlgo<T, U>` — generic over the element type `T` and a `SortLogger<T>` that captures every comparison, swap, and write. This dual-purpose design means the same code runs at full speed in benchmarks (with `NoOpLogger`) and produces frame-by-frame visualisation data (with `VisualizerLogger`).

## Registration

Sorts self-register at program startup — there is no central list to maintain. Two mechanisms coexist:

1. **`create_sort!` + `#[derive(SortRegistry)]`** — the older path. The macro generates a monomorphic wrapper, a `linkme` distributed-slice entry for benchmarks, and a `#[ctor]` hook for the runtime registry.
2. **`sort_family!`** — the newer declarative macro. Defines a variant tree of generic parameters and generates all combinations automatically. Used by merge sorts to register 80+ variants from one invocation.

## Migration status

Eight families are wired into the active dispatch (`mod.rs`): `merge_sorts`, `bubble_sorts`, `circle_sorts`, `comb_sorts`, `cycle_sorts`, `insertion_sorts`, `rod_sorts`, and `shell_sorts`. Of these, `merge_sorts` uses the `sort_family!` system; the others use `create_sort!` / distributed-slice registration.

The remaining families (`fun_sorts`, `heap_sort`, `quick_sorts`) are compiled but commented out, pending future work.

## Files in this directory

- `mod.rs` — top-level dispatch: routes a `choice` path to the correct family's `fn_sort`.
- `annotations.rs`, `example_generic_sort.rs` — reference/template files (disconnected).
