# fun_sorts

Experimental, educational, and intentionally-bad sorting algorithms. These exist for visualisation interest and algorithmic curiosity, not for practical use.

## Variants

- `slow_sort.rs` — **Slow sort**: multiply-and-surrender. Recursively finds the maximum by sorting both halves, taking the max of their maximums, then sorting `arr[..n-1]`. Worse than O(N^2) — truly slow by design.
- `slow_sort_potzko.rs` — **Slow sort (potzko)**: a variant of slow sort with different recursion shape.
- `stooge_sort.rs` — **Stooge sort**: recursively sorts the first 2/3, last 2/3, and first 2/3 again. O(N^(log3/log1.5)) ≈ O(N^2.71).
- `bad_heap_sort.rs` / `bad_heap_sort_alt.rs` — **Bad heap sort**: deliberately suboptimal heap-based sorts with distinctive failure-mode visualisations.
- `cyclent_sort.rs`, `cyclent_sort_opt.rs`, `cyclent_sort_stack.rs`, `cyclent_sort_stack_optimized.rs` — **Cyclent sort**: a cycle-sort-inspired experimental family with progressively-tightened variants (recursive, optimised, stack-based, stack-based optimised).
- `quick_surrender.rs` / `quick_surrender_optimised.rs` — **Quick surrender**: a quicksort parody that gives up easily and falls back to a simpler sort on small partitions.
- `random_shell_sort.rs` — **Random shell sort**: shell sort with randomly generated gap sequences. Different every run.

## Registration

Every file uses `sort_registry_macro::sort_family!` or `combo_codegen::family!` to publish itself into `bench_registry::ALGORITHMS`. They appear in the interactive menu under `sorts / fun sorts /`. Slow sorts that can't handle large random arrays use `register_test_cap!` to skip oversized inputs in the correctness battery.
