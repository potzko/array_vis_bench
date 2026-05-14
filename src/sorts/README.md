# sorts

All sorting algorithm implementations, organised by family. Each subfolder is
one family (e.g. `merge_sorts/`, `quick_sorts/`) containing the algorithm
variants and the macro invocations that publish them.

## What lives here

Sorts are the largest of six algorithm categories registered by this crate.
The other categories (rotations, partitions, merges, quick-selects,
small-sorts) follow the same shape but live under `utils/` or other
sub-trees of `sorts/`. See [bench_registry.rs](../bench_registry.rs) for
the `Category` enum and the per-category input registries.

## How a sort is registered

Every algorithm — sort or otherwise — ends up as one entry in the
`bench_registry::ALGORITHMS` distributed-slice. The benchmark binary, the
test harness, and the interactive visualiser all iterate that single slice.

Sorts publish themselves through `combo_codegen::family!`, a declarative
cross-product macro. One invocation describes the generic-parameter slots
of a sort and the concrete types each slot can take; the macro expands
into one `AlgorithmEntry` per leaf of the cross-product, plus the per-leaf
ctor that registers a navigation-tree path.

```rust
combo_codegen::family!(
    type = QuickSort<{P}, {V}, {SS}>,
    uses = [
        "crate::sorts::quick_sorts::quick_sort::QuickSort",
        "crate::sorts::quick_sorts::partitions::{Lomuto, Hoare, ThreeWay, Block}",
        "crate::sorts::quick_sorts::pivot_selectors::{FirstElement, MiddleElement, MedianOfThree, Ninther}",
        "crate::utils::small_sort::{NoSmallSort, InsertionSmallSort, Size2SmallSort}",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
    ],
    P:  Partition,
    V:  PivotSelector,
    SS: SmallSort,
    name        = "quick sort",
    big_o       = "O(N log N)",
    stable      = false,
    direct_sort = true,
    path        = ["quick sorts", "{P}", "{V}", "{SS}"],
);
```

Adding a new pivot strategy (or partition, or small-sort threshold) requires
nothing in this file: declare the new type, slap a
`combo_codegen::component!(PivotSelector, MyPivot, "label")` next to it, and
every family that takes a `PivotSelector` slot picks it up on the next
build.

## Where things live

- One folder per family. Inside each folder, the sort's struct, its generic
  parameter slots, and the `family!` invocation sit together.
- Cross-family component traits live in `utils/` (`utils/rotation/`,
  `utils/shell_sequences/`, `utils/shell_branching/`, `utils/small_sort.rs`)
  or alongside their primary consumer
  (`quick_sorts/partitions.rs`, `quick_sorts/pivot_selectors.rs`).
- Non-sort algorithm registration (rotations, partitions, merges,
  quick-selects, small-sorts) lives next to its sort family but emits
  entries with the matching `Category::*`. See for example
  `quick_sorts/partitions_standalone.rs` or
  `merge_sorts/standalone_registry.rs`.

## Top-level menu

When the interactive binary starts, every algorithm appears under one of
six top-level groups derived from its `Category`:

```
small-sorts     ← utils/small_sort.rs
rotations       ← utils/rotation/*
merges          ← merge_sorts/standalone_registry.rs
partitions      ← quick_sorts/partitions_standalone.rs
quick selects   ← quick_selects/standalone_registry.rs
sorts           ← every family in this folder
```

The "sorts" group expands into the per-family sub-tree (bubble sorts, quick
sorts, merge sorts, …); the others list their concrete variants directly.
