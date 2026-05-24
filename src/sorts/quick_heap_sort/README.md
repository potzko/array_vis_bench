# quick_heap_sort

Quick heap sort — a quicksort/heapsort hybrid. Partition the array around a pivot, then heap-sort one side and recurse into the other. Combines quicksort's good average-case constants with heapsort's bounded worst case.

## How it works

1. Pick a pivot and partition the array (any `PartitionScheme`).
2. The larger side becomes the heap region; build an implicit heap over it (any `HeapAlgorithm`).
3. Repeatedly extract the root of the heap into the smaller side's correct position.
4. Recurse into the smaller side.

The "deferred" variant skips the final small-region cleanup until the recursion unwinds, then runs one whole-array insertion pass.

## Files

- `quick_heap_sort.rs` — `QuickHeapSort<H: HeapAlgorithm, P, V, SS>` family entry.
- `deferred_quick_heap_sort.rs` — same shape but with deferred small-region completion.

## Cross-product slots

| Slot | Trait | Source |
|---|---|---|
| Partition scheme | `PartitionScheme` | `crate::sorts::quick_sorts::partitions` |
| Pivot selector | `PivotSelector` | `crate::sorts::quick_sorts::pivot_selectors` |
| Heap algorithm | `HeapAlgorithm` | `crate::sorts::heap_sort::heap_algorithm` |
| Small-sort cutoff | `SmallSort` / `DeferredSmallSort` | `crate::utils::small_sort` |

## Registration

Both files use `combo_codegen::family!`. State for the inner heap is allocated explicitly using `HeapAlgorithm::new_state` / `drop_state` so the visualiser sees the heap's aux-array lifetime as a single region.
