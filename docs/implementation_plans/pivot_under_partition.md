# Implementation plan: move pivot under the partition (quicksort family re-model)

Status: **DESIGNED / IDEA, NOT STARTED.** Decided direction 2026-06-17. Separate concern from
the enumeration strategies ([enumeration_strategies.md](enumeration_strategies.md)), but they
compose nicely (this makes the rest-fill sugar + arity fall out structurally).

## The change

Today the pivot is a **peer type-param of the sort**:
```
QuickSort<P: PartitionScheme, V: PivotInput, SS: SmallSort>   // pivot V sits beside the partition P
DeferredQuickSort<P, V, DSS>
```
and partitions take pivot **indices at runtime** (`PartitionScheme::partition(arr, pivots: &[usize], …)`,
`const N_PIVOTS`). So `QuickSort` drives pivot selection (`V` computes indices) and hands them to `P`.

The pivot belongs to the *partition*, not the sort. Quicksort should just be an **indirection over a
self-pivoting partition**:
```
QuickSort<Partition, SmallSort>     // no pivot param
DeferredQuickSort<Partition, SmallSort>
```
- a **pivot-based** partition owns its selector — e.g. `Pivoted<LeftLeftPartition, FirstElement>`
  (a wrapper that picks pivots via the selector, then runs the scheme).
- a **pivotless** partition (`HeapExtractPartition`) has no pivot at all and no longer has to fake one.

## Why

- Matches the semantics: pivot is a property of pivot-based partitions, period.
- `HeapExtractPartition` stops pretending to carry a pivot (the thing that motivated this).
- The cross-slot **arity coupling disappears**: today `project pivot PivotSingle/Dual/None` +
  `Partition[pivot = p]` + the shared `p`/`part` helper bindings exist only because pivot lives one
  level too high. With pivot under the partition, arity is **structural** — a single-pivot scheme's
  wrapper only accepts single selectors by its type bound; a pivotless partition simply has no pivot
  hole. No projection, no refinement, no `QSPivotSingle/Dual/None` roles.
- Makes the query clean and the rest-fill sugar honest: `quick_sort<partition = LL<pivot = _>, small = _>`
  nests pivot naturally; `quick_sort<_>` just fills partition+small.

## Affected surface

- **`quick_sort_lib`**: `QuickSort<P, V, SS>` → `QuickSort<Part, SS>`; same for `DeferredQuickSort`
  and the `HasTimeBounds/HasSpace/HasStability` impls (drop the `V` param, read complexity from
  `Part`).
- **New `Pivoted<Scheme, Selector>`** partition wrapper (likely in `quick_sort_lib` or a small crate):
  implements a "self-pivoting partition" interface QuickSort calls; internally uses `Selector` to pick
  pivots then runs `Scheme::partition`. Bound enforces `Selector::N == Scheme::N_PIVOTS` (this is where
  the old arity coupling now lives — in the type, not the query). Less invasive than making every
  scheme crate generic.
- **Partition crates** (`partition_lomuto/hoare/three_way/block`): unchanged (still `PartitionScheme`
  taking indices); they get wrapped by `Pivoted`. `HeapExtractPartition` already self-contained →
  implements the self-pivoting interface directly.
- **`quick.spec`**: `quick_sort` driver drops the `pivot` slot. Pivot-based partition components become
  `pivoted<scheme, pivot>` (a pivot sub-slot); heap-extract has no pivot slot. Delete the
  `project pivot …` lines and the query's `p`/`part` shared-var coupling.
- **Consistency follow-on** (decide separately): `quick_select_lib`'s `RecursiveQuickSelect<P, V>` /
  `IterativeQuickSelect<P, V>` and `fun_sorts` `quick_surrender` use the same `P, V` shape +
  the `QSPivotSingle/Dual/None` coupling I built in Phase 1. Re-modeling those the same way would
  delete that coupling too — but it's a bigger blast radius; do quick_sort first, then decide.

## Risk / open questions

- **Medium-high Rust refactor.** Every `QuickSort<…>` instantiation re-threads; the dual-pivot
  `CombinedSelector<a, b>` now nests under `Pivoted`. Monomorphisation/perf unchanged
  (`Pivoted<LL, FirstElement>` is still one concrete type).
- `Pivoted<Scheme, Selector>` wrapper vs making each scheme generic over its selector — wrapper is
  less invasive; confirm it composes with the dual-pivot partition (`DualPivotPartition`, `N_PIVOTS=2`).
- Apply to quick_select / quick_surrender now (consistency) or stage it?
- Emitted variant **names/labels** will shift (pivot now renders under the partition) — update the
  count tests + any name-pinned assertions.

## Pointers
- `crates/families/quick_sort_lib/src/quick_sort.rs` (`QuickSort<P, V, SS>`),
  `deferred_quick_sort.rs`; `array_vis_bench_traits/src/role/partition.rs` (`PartitionScheme`,
  `N_PIVOTS`); `crates/partitions/*`; `crates/families/quick_sort_lib/quick.spec`.
- Related: Phase-1 quick-select coupling in `project_first_class_kinds` (memory) +
  `crates/families/quick_select_lib/quick_select.spec`.
