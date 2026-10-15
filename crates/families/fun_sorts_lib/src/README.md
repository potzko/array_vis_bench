# `src`

Source of `fun_sorts_lib`. The crate README one level up says what it is for.

- `bad_heap_sort.rs`: "Bad heap" sort, recursive variant.
- `bad_heap_sort_alt.rs`: "Bad heap" sort, heap-extract variant.
- `composable.rs`: Compositional complexity annotations for the fun-sorts types.
- `cyclent_sort.rs`: Cyclent sort, plain variant (v0).
- `cyclent_sort_opt.rs`: Cyclent sort, single-bound optimized variant (v1).
- `cyclent_sort_stack.rs`: Cyclent sort, iterative quicksort (v3).
- `cyclent_sort_stack_optimized.rs`: Cyclent sort, stack-optimized variant (v1).
- `lib.rs`: Fun sorts, a grab bag of pedagogical and adversarial algorithms.
- `quick_surrender.rs`: Quick surrender, selection sort via quickselect.
- `quick_surrender_optimised.rs`: Quick surrender (optimised), block quickselect plus small sort.
- `random_shell_sort.rs`: Random shell sort, shell sort with a randomly-generated gap sequence.
- `slow_sort.rs`: Slow sort, the "multiply and surrender" anti-sort.
- `slow_sort_potzko.rs`: the crate's source.
- `stooge_sort.rs`: Stooge sort, recursive three-way swap-and-recurse.
