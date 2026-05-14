# array_vis_bench

A benchmark and visualisation framework for ordered-data algorithms — sorts
and their building blocks (rotations, partitions, merges, quick-selects,
small-sorts). Cross-products of generic parameter slots are auto-enumerated
into one flat registry; the same registry feeds the interactive
visualiser, the speed-test binary, and the correctness test harness.

## Binaries

- **`array_vis_bench`** (default) — interactive visualiser. Walks a
  category → family → variant menu, asks for an input shape and array
  size, then renders an MP4 of the resulting `SortLog` stream.
- **`speed_test`** (`cargo run --bin speed_test`) — runs every registered
  algorithm against the primary input for its category and prints
  median wall-clock times, fastest-first.
- **Benchmarks** (`cargo bench --bench sorts`) — Criterion benchmarks with
  adaptive thresholding for the slow algorithms.

## Algorithm categories

Every registered algorithm belongs to one of six categories
(`bench_registry::Category`). The top-level menu groups by category:

| Category    | Where it's registered                                  |
|-------------|--------------------------------------------------------|
| sorts       | every family folder in `src/sorts/`                    |
| rotations   | `src/utils/rotation/`                                  |
| partitions  | `src/sorts/quick_sorts/partitions_standalone.rs`       |
| merges      | `src/sorts/merge_sorts/standalone_registry.rs`         |
| quick-selects | `src/sorts/quick_selects/standalone_registry.rs`     |
| small-sorts | `src/utils/small_sort.rs`                              |

Inputs are also per-category: `bench_registry::SORT_INPUTS`,
`ROTATION_INPUTS`, etc. Each input registry has exactly one entry marked
`primary: true` — the default the harness picks when nothing is specified.

## Project structure

```
src/
  sorts/            — sort algorithm implementations, organised by family
    bubble_sorts/   — bubble, shaker, odd-even
    beap_sort/      — beap (bi-parental) heap sort + quick-build variants
    circle_sorts/   — recursive and bottom-up circle sorts
    comb_sorts/     — comb sort, parameterised over shrink ratio
    cycle_sorts/    — write-optimal cycle sort
    fun_sorts/      — slow sort, stooge sort, quick surrender, …
    heap_sort/      — N-ary heap sort + quick-build variants
    insertion_sorts/ — baseline insertion sort
    merge_sorts/    — merge sort variants + standalone merge algorithms
    quick_sorts/    — quicksort + standalone partition algorithms
    quick_selects/  — quickselect (single and dual pivot) + standalone
    quick_heap_sort/ — quick-heap hybrid sorts
    rod_sorts/      — rod sort, parameterised over branching strategy
    shell_sorts/    — shell sort + shell-shell sort, parameterised over
                       gap sequence
    weak_heap_sort/ — weak heap sort
  utils/            — shared building blocks
    rotation/       — 11 array rotation algorithms (reversal, juggling,
                       trinity, drill, helix, piston, grail, gries-mills,
                       bridge, contrev, auxiliary)
    shell_sequences/ — gap sequence generators (Knuth, Sedgewick, Ciura,
                       Pratt, …)
    shell_branching/ — branching strategies for shell-shell / rod sort
    small_sort.rs   — small-sort threshold strategies
  traits/           — SortLogger re-exports + the legacy SortAlgo trait
  visualise/        — MP4 rendering bridge (delegates to sort_vis crate)
  inputs.rs         — per-category input definitions (shuffled, ascending,
                       all-same, …)
  bench_registry.rs — Category enum, ALGORITHMS distributed slice,
                       per-category input slices and dispatchers
  main.rs           — interactive CLI
  bin/speed_test.rs — speed comparison binary
benches/
  sorts.rs          — Criterion benchmark with adaptive thresholding
```

### Helper crates

| Crate | Purpose |
|---|---|
| `sort_logger` | `SortLogger<T>` trait, `SortLog<T>` event enum, `NoOpLogger`, `VisualizerLogger`. |
| `sort_vis` | MP4 / framebuffer renderer. Replays a `Vec<SortLog>` into an animation. |
| `sort_registry_core` | Navigation-tree builder (`SortTree`) and the per-leaf `register_sort_path` hook. |
| `sort_registry_macro` | `sort_family!` proc-macro that expands a declarative cross-product into one `AlgorithmEntry` per leaf. |
| `combo_codegen` | Build-script driver that scans `component!` and `family!` invocations and emits the generated combinations files under `OUT_DIR`. |

## Self-registration

There is no central list to maintain. Every algorithm declares itself via
one of:

- **`combo_codegen::family!`** — declarative variant-tree macro. One
  invocation per family enumerates every cross-product of generic slot ×
  concrete type into the `bench_registry::ALGORITHMS` distributed-slice.
- **Per-category registration macros** — `register_rotation!`,
  `register_partition!`, `register_merge!`, `register_aux_merge!`,
  `register_quick_select_single!`, `register_quick_select_dual!`,
  `register_small_sort!`. Each is the non-sort sibling of `family!`,
  emitting an `AlgorithmEntry` with the matching `Category::*`. The
  hand-rolled cross-products invoke them once per leaf.

Both paths end with the same per-leaf `#[ctor]` that calls
`sort_registry_core::register_sort_path` so the variant shows up in the
interactive menu tree.

See `docs/` for detailed architecture documentation.

## Note for LLMs

If you are an LLM working on this project, read `docs/llm/` before making
changes. It contains a context-loading guide, key files to always index,
patterns to follow, and instructions for keeping documentation up to date.
