# array_vis_bench

A sorting algorithm benchmark and visualisation framework. Implements 50+ sorting algorithms across 11 families, with animated GIF visualisation and Criterion-based benchmarking.

## Binaries

- **`array_vis_bench`** (default) — interactive visualiser. Presents a tree menu to pick a sort, array type (random/ascending/descending), and size. Runs the sort with a `VisualizerLogger` that captures every comparison and swap, then renders an animated GIF.
- **`speed_test`** (`cargo run --bin speed_test`) — lightweight speed comparison. Runs every registered sort against a shuffled array, measures median wall-clock time, prints results fastest-first. No Criterion overhead.
- **Benchmarks** (`cargo bench --bench sorts`) — full Criterion benchmarks with adaptive thresholding that drops slow sorts at larger array sizes.

## Project structure

```
src/
  sorts/            — sort algorithm implementations, organised by family
    merge_sorts/    — the actively developed family (sort_family! system)
    shell_sorts/    — shell sort, shell-shell sort, gap sequences
    quick_sorts/    — quicksort partition/pivot strategy variants
    bubble_sorts/   — bubble, shaker, odd-even
    circle_sorts/   — recursive and bottom-up circle sorts
    comb_sorts/     — comb sort with configurable shrink factors
    cycle_sorts/    — write-optimal cycle sort
    heap_sort/      — binary/ternary/n-ary heaps, heap-quick hybrids
    insertion_sorts/ — baseline insertion sort
    rod_sorts/      — rod sort (branching-strategy parameterised)
    fun_sorts/      — slow sort, stooge sort, quick surrender, etc.
    merge_sorts_old/ — legacy hand-written merge sorts (superseded)
  utils/            — shared building blocks
    rotation/       — 11 array rotation algorithms (reversal, juggling, trinity, etc.)
    shell_sequences/ — gap sequence generators (Knuth, Sedgewick, Ciura, Pratt, etc.)
    shell_branching/ — branching strategies for shell-shell/rod sort
  traits/           — SortAlgo trait, global registries, create_sort! macro
  rotations/        — convenience re-exports from utils/rotation
  visualise/        — GIF rendering bridge (delegates to sort_vis crate)
  sort_test/        — correctness validation harness
  bench_registry.rs — linkme distributed slice for benchmark entries
  main.rs           — interactive CLI with tree-based sort selection
  bin/speed_test.rs — lightweight speed comparison binary
benches/
  sorts.rs          — Criterion benchmark with adaptive thresholding
```

### Helper crates

| Crate | Purpose |
|---|---|
| `sort_logger` | `SortLogger<T>` trait + `NoOpLogger`/`VisualizerLogger`. Zero-cost instrumentation for every sort operation. |
| `sort_vis` | GIF renderer. Replays a `Vec<SortLog>` into an animated image. |
| `sort_registry_core` | Sort metadata registry and navigation-tree builder (`SortTree`). |
| `sort_registry_macro` | `#[derive(SortRegistry)]` and `sort_family!` proc macros for automatic registration. |

## Sort self-registration

Sorts register themselves at program startup — there is no central list to maintain. Two mechanisms coexist:

1. **`create_sort!` + `#[derive(SortRegistry)]`** — generates a monomorphic wrapper, a `linkme` distributed-slice entry for benchmarks, and a `#[ctor]` hook for the runtime registry.
2. **`sort_family!`** — declarative variant-tree macro. Defines generic slots and their concrete types, generates all combinations automatically. Used by merge sorts to register 80+ variants from one invocation.

See `docs/` for detailed architecture documentation.

## Note for LLMs

If you are an LLM working on this project, read `docs/llm/` before making changes. It contains a context-loading guide, key files to always index, patterns to follow, and instructions for keeping documentation up to date.
