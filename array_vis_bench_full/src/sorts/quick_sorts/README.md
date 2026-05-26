# quick_sorts

Quick sort variants — every meaningful cross-product of pivot selector × partition scheme × small-sort cutoff, plus dual-pivot and deferred-small-sort flavours.

## How quick sort works

Quick sort picks a *pivot*, *partitions* the array so everything `< pivot` ends up on the left and everything `> pivot` on the right, then recurses on each side. The choice of pivot and partition scheme controls worst-case behaviour, constant factors, and stability.

## Variant axes

| Axis | Provided by | Implementations |
|---|---|---|
| Pivot selector (`V: PivotSelector`) | `pivot_selectors.rs` | `FirstElement`, `MiddleElement`, `LastElement`, `MedianOfThree`, `MedianOfMedians`, `Ninther` |
| Dual-pivot selector (`DPS: DualPivotSelector`) | `pivot_selectors.rs` | `CombinedSelector<V1, V2>` (cross-product of two `PivotSelector`s) plus the native `NintherDualPivot` (samples 9 positions, returns the 1/3 and 2/3 quantiles) |
| Partition scheme (`P: PartitionScheme`) | `partitions.rs` | `Lomuto`, `Hoare`, `ThreeWay` (Dutch National Flag), `Block` (branchless batched), `MovingPivot` |
| Small-sort cutoff (`SS: SmallSort`) | `crate::utils::small_sort` | `NoSmallSort`, `Size1SmallSort`, `Size2SmallSort`, `InsertionSmallSort<N, S>`, `NetworkSmallSort`, `Network16SmallSort` |

The cross-products live in four family files:

- `quick_sort.rs` — `QuickSort<P, V, SS>`: standard, classic small-sort applied at recursion base.
- `deferred_quick_sort.rs` — `DeferredQuickSort<P, V, DSS>`: recurses only until segments fall below the cutoff, then runs one final insertion-sort pass over the whole array.
- `dual_pivot_quick_sort.rs` — `DualPivotQuickSort<DPS, SS>`: Yaroslavskiy-style three-way partition between two pivots.
- `deferred_dual_pivot_quick_sort.rs` — dual-pivot with the deferred-small-sort strategy.

Each file's `combo_codegen::family!` invocation lists every concrete instantiation that should be generated; the build script writes them into `OUT_DIR/quick_sorts_combinations.rs`, which `mod.rs` then includes.

## Supporting files

- `pivot_selectors.rs` — `PivotSelector` and `DualPivotSelector` traits + all implementations. Each concrete type is annotated with `combo_codegen::component!(PivotSelector, …)` so it is picked up automatically by the codegen.
- `partitions.rs` — `PartitionScheme` trait + all implementations.
- `partitions_standalone.rs` — registers every `(PartitionScheme × PivotSelector)` pair as a `Category::Partition` standalone algorithm so the visualiser and bench harness can drive partitions directly, not just through a containing quick sort.
- `mod.rs` — module declarations + `include!` of the generated combinations file.

## Registration

All variants register themselves into `bench_registry::ALGORITHMS` via `combo_codegen::family!`. No central list to edit. Adding a new pivot selector is two lines: declare the type and slap a `combo_codegen::component!(PivotSelector, MyPivot, "label")` next to it — every family that takes a `PivotSelector` slot picks it up on the next build.
