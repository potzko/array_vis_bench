# Architecture Overview

## Crate dependency graph

```
                    sort_registry_macro (proc-macro, legacy sort_family!)
                            │
                            ▼
                    combo_codegen (build-script scanner + family! / component! markers)
                            │
                            ▼
array_vis_bench ──────► sort_registry_core (navigation-tree metadata)
  (root crate)  │
                ├─────► sort_logger (SortLogger trait, NoOpLogger, VisualizerLogger)
                │
                └─────► sort_vis (MP4 / GIF renderer)
```

`sort_logger` is the lightest dependency — only the logger trait and the no-op / visualiser implementations. `sort_vis` (which pulls in `image`) is only needed by the visualiser binary. `combo_codegen` is build-time only; the scanned cross-products are emitted into `$OUT_DIR/<family>_combinations.rs` and included from each family's `mod.rs`.

## Data flow

### One slice, three consumers

```
            bench_registry::ALGORITHMS  (linkme distributed-slice)
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   visualiser     bench harness   correctness suite
  (main.rs)      (benches/sorts) (per-leaf #[test] +
                                  all_registered_algorithms_are_correct)
```

`AlgorithmEntry` carries the entry's `name`, `category`, `big_o`, `stable`, optional `max_input_size`, and two function pointers: `run_with_input(input_name, config, logger)` and `run_correctness()`. Every consumer uses the same shape regardless of category — the category enum exists only for menu grouping and per-category input dispatch.

### Benchmark path

```
ALGORITHMS slice
    │
    ▼
benches/sorts.rs
    │
    ▼
entry.run_with_input(primary_input, RunConfig, &mut NoOpLogger)
    │
    ▼
NoOpLogger — every logger method compiles away entirely
```

Function pointers in `AlgorithmEntry` take `&mut dyn SortLogger<usize>` so the harness can swap loggers without type-erasing the algorithm. With `NoOpLogger` behind the trait object the inner loop is a single virtual call per operation, but `NoOpLogger`'s methods are trivial and easily inlined behind it.

### Visualisation path

```
main.rs
    │
    ▼
SortTree navigation → entry name
    │
    ▼
bench_registry::ALGORITHMS.find(name)
    │
    ▼
entry.run_with_input(input_name, config, &mut VisualizerLogger)
    │
    ▼
VisualizerLogger captures SortLog<usize> events into a Vec
    │
    ▼
sort_vis::render_gif(original_arr, &log) → output.mp4 / .gif
```

The `VisualizerLogger` records every comparison, swap, write, and auxiliary allocation. `bench_registry::emit_init_events` emits the `CreateAuxArrT + SetScale + N×WriteData` prelude so the log alone fully describes the initial state.

### Correctness path

```
#[test] all_registered_algorithms_are_correct
    │
    ▼
for entry in ALGORITHMS:
    spawn subprocess(self, env: AVB_RUN_CHECK_SORT=entry.name)
        │
        ▼
    ctor::subprocess_dispatch picks up the env var,
    calls entry.run_correctness(), exits
```

The subprocess always re-executes the same binary, so a freshly-added algorithm is immediately picked up without rebuilding a separate runner. `run_correctness` invokes the category's battery: `sort_battery` / `rotation_battery` / `partition_battery` / `merge_battery` / `quick_select_battery` / `small_sort_battery`. Each verifies category-specific shape (sortedness + permutation, rotation contract, partition split, etc.).

## Registration flow

At program startup (before `main`):

1. **Link time** — `linkme` collects every `#[distributed_slice(ALGORITHMS)]` static across compilation units.
2. **`#[ctor]` hooks** — each algorithm's per-leaf ctor calls `sort_registry_core::register_sort_path` so the variant joins the navigation tree.
3. **`bench_registry::validate_at_startup`** — runs once, before `main`. Checks: duplicate algorithm names within a category, duplicate tree paths, missing / multiple primary inputs per category, empty input registries for non-empty categories. Any inconsistency panics with a precise message.

## Algorithm interface

Algorithms expose an inherent `sort` (or category-equivalent) function:

```rust
impl QuickSort<P, V, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // ...
    }
}
```

- `T` — the element type. Always `usize` for the registry harness, but the body is generic.
- `U: ?Sized` — the logger. Same code path runs with `NoOpLogger` (bench), `VisualizerLogger<usize>` (visualisation), or `dyn SortLogger<usize>` (registry-driven).

This means the same algorithm code services bench and viz with no runtime overhead in the bench path.

## Parameterised families

| Family | Slots |
|---|---|
| Shell sort | `Seq: GapSequence` |
| Shell-shell sort / rod sort | `S: BranchingStrategy` |
| Top-down / bottom-up merge sort | `SS: SmallSort`, `const PING_PONG: bool`, `const EARLY_EXIT: bool` |
| Rotation merge sort | `SS: SmallSort`, `M: RotationMerge<R>` (with `R: Rotation`), `const EARLY_EXIT: bool` |
| Natural merge sort | `const PING_PONG: bool`, `const EARLY_EXIT: bool` |
| Quick sort | `P: PartitionScheme`, `V: PivotSelector`, `SS: SmallSort` |
| Dual-pivot quick sort | `DPS: DualPivotSelector`, `SS: SmallSort` |
| Deferred quick sort | `P`, `V`, `DSS: DeferredSmallSort` |

All slots are resolved at compile time via generics or const generics. `combo_codegen::family!` enumerates the cross-product and emits one `AlgorithmEntry` per leaf.

## Module layout principles

- **One concept per folder** — each sort family, utility category, and strategy set gets its own folder with a README explaining the concept.
- **Registration is local** — every family registers itself via `family!` / `register_*!` macros next to the implementation. There is no central list of algorithm names.
- **Validation is central** — `validate_at_startup` enforces registry invariants once, in one place.
