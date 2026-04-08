# Architecture Overview

## Crate dependency graph

```
                    sort_registry_macro (proc-macro)
                            │
                            ▼
array_vis_bench ──────► sort_registry_core (metadata + tree)
  (root crate)  │
                ├─────► sort_logger (SortLogger trait, NoOpLogger, VisualizerLogger)
                │
                └─────► sort_vis (GIF renderer)
```

`sort_logger` is the lightest dependency — just the trait and two implementations. Every sort depends on it. `sort_vis` is the heaviest (pulls in the `image` crate) and is only needed by the visualiser binary.

## Data flow

### Benchmark path

```
BENCH_SORTS (linkme slice)
    │
    ▼
criterion harness (benches/sorts.rs)
    │
    ▼
entry.run(arr)  ──►  sort_fn(arr: &mut [usize])
                         │
                         ▼
                     NoOpLogger  (all log() calls compile away)
```

Function pointers are monomorphic `fn(&mut [usize])` — no trait objects, no vtable dispatch. The compiler can inline the entire sort and optimise aggressively.

### Visualisation path

```
main.rs
    │
    ▼
select_sort() → sort_name (via SortTree navigation)
    │
    ▼
create_sort_choice(sort_name) → choice path (e.g. ["merge_sorts", "..."])
    │
    ▼
visualise_sort(arr, logger, choice)
    │
    ├─► sorts::fn_sort(arr, logger, choice)  ──►  sort runs with VisualizerLogger
    │                                                    │
    │                                                    ▼
    │                                              logger.log: Vec<SortLog<usize>>
    │
    └─► sort_vis::render_gif(original_arr, base_ptr, &log)
            │
            ▼
        output.mp4 (animated GIF on disk)
```

The `VisualizerLogger` captures every comparison, swap, write, and auxiliary allocation as a `SortLog` enum variant. The renderer replays these events to generate frames.

## Registration flow

At program startup (before `main`):

1. **Link time** — `linkme` collects all `#[distributed_slice(BENCH_SORTS)]` statics across compilation units into a single array.
2. **`#[ctor]` hooks** — each sort's constructor runs, inserting function pointers into `SORT_REGISTRY` (benchmark dispatch) and `SORT_VIS_REGISTRY` (visualisation dispatch), and metadata into `sort_registry_core`'s `SORT_ENTRIES`.
3. **`main` startup** — `validate_sort_routing()` checks that every registered sort has a visualisation dispatch route, panicking immediately if one is missing.

## Generic sort design

Sorts are generic over two type parameters:

```rust
trait SortAlgo<T: Ord + Copy, U: SortLogger<T>> {
    fn sort(arr: &mut [T], logger: &mut U);
    // ...
}
```

- `T` — the element type. Usually `usize` for benchmarks, but the sort works with any `Ord + Copy` type.
- `U` — the logger type. `NoOpLogger` for benchmarks (zero-cost), `VisualizerLogger` for recording, `dyn SortLogger<T>` for dynamic dispatch.

This design means the same algorithm code is used for both benchmarking and visualisation, with no runtime overhead in the benchmark path.

## Parameterised sorts

Many sort families are parameterised over strategy traits:

| Sort family | Parameters |
|---|---|
| Shell sort | `Seq: GapSequence` |
| Shell-shell sort | `S: BranchingStrategy` |
| Rod sort | `S: BranchingStrategy` |
| Merge sort (aux) | `S: SmallSort`, `PING_PONG: bool`, `EARLY_EXIT: bool` |
| Merge sort (rotation) | `S: SmallSort`, `M: RotationMerge`, `EARLY_EXIT: bool` |
| Rotation merge | `R: Rotation` |
| Quick sort (generic) | Pivot strategy, Partition strategy |

All parameters are resolved at compile time via generics/const generics. The `sort_family!` macro generates all meaningful combinations and registers each as a separate sort entry.

## Module layout principles

- **One concept per folder** — each sort family, utility category, and strategy set gets its own folder with a README explaining the concept.
- **Registration is local** — each sort family registers itself via `combinations.rs` or `#[derive(SortRegistry)]`. No central list.
- **Legacy code is preserved** — `merge_sorts_old/`, `classic_shell_sorts/` contain the original hand-written versions before the generic system was built. Kept for reference and comparison.
