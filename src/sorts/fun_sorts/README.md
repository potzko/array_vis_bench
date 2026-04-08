# fun_sorts

Experimental, educational, and intentionally-bad sorting algorithms. These exist for visualisation interest and algorithmic curiosity, not for practical use.

## Variants

- `slow_sort.rs` — **Slow sort**: multiply-and-surrender. Recursively finds the maximum by sorting both halves, taking the max of their maximums, then sorting `arr[..n-1]`. Worse than O(N^2) — truly slow by design.
- `stooge_sort.rs` — **Stooge sort**: recursively sorts the first 2/3, last 2/3, and first 2/3 again. O(N^(log3/log1.5)) ≈ O(N^2.71). Famously inefficient.
- `bad_heap_sort.rs` / `bad_heap_sort_alt.rs` — **Bad heap sort**: deliberately suboptimal heap-based sorts. Interesting failure modes for visualisation.
- `cyclent_sort.rs` — **Cyclent sort**: a cycle-sort-inspired experimental algorithm.
- `cyclent_sort_stack.rs` / `cyclent_sort_stack_optimized.rs` — Stack-based cyclent sort variants with increasing optimisation.
- `quick_surrender.rs` / `quick_surrender_optimised.rs` — **Quick surrender**: a quicksort parody that gives up easily. Partition + fall back to simpler sort on small partitions with humorous thresholds.
- `random_shell_sort.rs` — **Random shell sort**: shell sort with randomly generated gap sequences. Different every run — useful for exploring how random gaps compare to carefully chosen ones.

## Status

Compiled but commented out of the active dispatch in `sorts/mod.rs`. These are not expected to be migrated to `sort_family!` — they exist for fun and visualisation.
