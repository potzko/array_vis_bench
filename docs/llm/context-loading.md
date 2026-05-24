# Context Loading Guide

How to efficiently load enough context to work on this codebase.

## Always read these files first

These files define the abstractions that everything else depends on. Load them into context before doing any work:

| File | Why |
|---|---|
| `sort_logger/src/sort_logger.rs` | **The most important file.** Defines `SortLogger<T>` — every sort calls these methods. You cannot write or modify a sort without understanding `cond_swap_lt`, `write_data`, `cmp_le_accross`, `create_aux_arr_t`, etc. |
| `sort_logger/src/sort_log.rs` | The `SortLog<T>` enum — all event types that sorts can emit. |
| `src/bench_registry.rs` | The single source of truth: `AlgorithmEntry`, `Category`, `ALGORITHMS` distributed-slice, the per-category correctness batteries, and the subprocess test-dispatch ctor. Every algorithm — sort, rotation, partition, merge, quick-select, small-sort — ends up as one entry here. |
| `src/traits/sort_traits.rs` | `SortAlgo<T, U>` — the legacy 4-method trait. Still used by hand-rolled callers but no longer the registration mechanism. |
| `src/traits/mod.rs` | Thin re-export shim plus the `SortFn` alias (`fn(&mut [usize], &mut NoOpLogger)`). The legacy `SORT_REGISTRY` / `SORT_VIS_REGISTRY` / `create_sort!` are gone. |

## Read these when working on specific areas

| Area | Files to load |
|---|---|
| **Adding/modifying a sort** | The sort's own file + its family's `mod.rs` + `combinations.rs` (if it has one) |
| **Rotation algorithms** | `src/utils/rotation/mod.rs` (trait + helpers) + the specific rotation file |
| **Shell sort gap sequences** | `src/utils/shell_sequences/mod.rs` |
| **Branching strategies** | `src/utils/shell_branching/mod.rs` |
| **Merge sort infrastructure** | `src/sorts/merge_sorts/small_sort.rs`, `rotation_merge.rs`, `rotation.rs`, `utils.rs` |
| **Registration system** | `combo_codegen/src/lib.rs` (build-time scanner + cross-product `family!` / `component!` markers) + `sort_registry_core/src/lib.rs` (tree builder). The legacy `sort_registry_macro/src/lib.rs` houses the older single-leaf `sort_family!` proc-macro. |
| **`combo_codegen::family!` examples** | `src/sorts/quick_sorts/quick_sort.rs`, `src/sorts/merge_sorts/top_down.rs`, `src/sorts/merge_sorts/rotation.rs` |
| **Benchmarks** | `benches/sorts.rs` + `src/bench_registry.rs` |
| **Visualisation** | `src/visualise/mod.rs` + `sort_vis/src/lib.rs` |
| **Main CLI** | `src/main.rs` |

## Reading the folder READMEs

Every folder has a `README.md` explaining what it contains and why. When you're unfamiliar with a folder, read its README first — it will tell you the concept, the design decisions, and what each file does. This is faster than reading every source file.

## What you can skip

- `target/` — build artifacts, never read these.
- `sort_logger/src/loggers.rs` — trivial implementations, only 30 lines.
- Auto-generated combination files under `$OUT_DIR/*_combinations.rs` — produced by the `combo_codegen` build script. Read the `family!` invocation instead.
- Individual sort files you're not modifying — the family README describes them.
