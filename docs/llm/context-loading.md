# Context Loading Guide

How to efficiently load enough context to work on this codebase.

## Always read these files first

These files define the abstractions that everything else depends on. Load them into context before doing any work:

| File | Why |
|---|---|
| `sort_logger/src/sort_logger.rs` | **The most important file.** Defines `SortLogger<T>` — every sort calls these methods. You cannot write or modify a sort without understanding `cond_swap_lt`, `write_data`, `cmp_le_accross`, `create_aux_arr_t`, etc. |
| `sort_logger/src/sort_log.rs` | The `SortLog<T>` enum — all event types that sorts can emit. |
| `src/traits/sort_traits.rs` | `SortAlgo<T, U>` — the trait every sort implements. |
| `src/traits/mod.rs` | `SORT_REGISTRY`, `SORT_VIS_REGISTRY`, `SortFn`/`SortVisFn` type aliases, and the `create_sort!` macro. |

## Read these when working on specific areas

| Area | Files to load |
|---|---|
| **Adding/modifying a sort** | The sort's own file + its family's `mod.rs` + `combinations.rs` (if it has one) |
| **Rotation algorithms** | `src/utils/rotation/mod.rs` (trait + helpers) + the specific rotation file |
| **Shell sort gap sequences** | `src/utils/shell_sequences/mod.rs` |
| **Branching strategies** | `src/utils/shell_branching/mod.rs` |
| **Merge sort infrastructure** | `src/sorts/merge_sorts/small_sort.rs`, `rotation_merge.rs`, `rotation.rs`, `utils.rs` |
| **Registration system** | `sort_registry_macro/src/lib.rs` + `sort_registry_core/src/lib.rs` |
| **sort_family! macro** | `sort_registry_macro/src/sort_family.rs` + any family's `combinations.rs` for usage examples |
| **Benchmarks** | `benches/sorts.rs` + `src/bench_registry.rs` |
| **Visualisation** | `src/visualise/mod.rs` + `sort_vis/src/lib.rs` |
| **Main CLI** | `src/main.rs` |

## Reading the folder READMEs

Every folder has a `README.md` explaining what it contains and why. When you're unfamiliar with a folder, read its README first — it will tell you the concept, the design decisions, and what each file does. This is faster than reading every source file.

## What you can skip

- `target/` — build artifacts, never read these.
- `sort_logger/src/loggers.rs` — trivial implementations, only 30 lines.
- `src/rotations/` — just re-exports from `src/utils/rotation/`, no logic.
- `merge_sorts_old/` — legacy code, only relevant if comparing old vs new implementations.
- Individual sort files you're not modifying — the family README describes them.
