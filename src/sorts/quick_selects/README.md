# quick_selects

Quick-select algorithms — find the k-th order statistic of an unsorted array by quicksort-style partitioning that only recurses into the side containing the target index. Average O(N), worst-case O(N²) without a good pivot strategy.

## How quick-select works

1. Pick a pivot from `arr`.
2. Partition `arr` so everything `< pivot` lands on the left and everything `> pivot` on the right.
3. If the pivot's final index is the target, we're done. If the target is smaller, recurse into the left partition; otherwise the right.

Only one side is recursed into, so the recurrence is `T(N) = T(N/2) + O(N) = O(N)` on average.

## Files

- `quick_select.rs` — `QuickSelect<P, V>`, generic over `PartitionScheme` and `PivotSelector` (reuses the slots from `crate::sorts::quick_sorts`).
- `dual_pivot_quick_select.rs` — Yaroslavskiy-style quick-select with two pivots; the target index decides which of the three regions to recurse into.
- `standalone_registry.rs` — registers every concrete `(P × V)` and dual-pivot variant as a `Category::QuickSelect` algorithm. The harness drives it via the `QuickSelectInput` registry (which yields both the array and a target index per case).

## Correctness

`bench_registry::correctness::quick_select_battery` verifies the post-condition: the value at `arr[target]` equals the `target`-th element of a fully sorted reference, every element to the left is `≤` it, every element to the right is `≥` it, and the result is a permutation of the input. Tests several shapes (reverse-sorted, random, duplicates, all-equal, sorted) and multiple target positions per shape.
