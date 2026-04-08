# quick_heap_sorts

Hybrid sorts that combine heap and quicksort ideas.

## Concept

These algorithms use quicksort-style partitioning to divide the array, but incorporate heap operations to handle one or both sides of the partition. The goal is to combine quicksort's good average case with heap sort's guaranteed O(N log N) worst case.

## Variants

- `heap_quick_sort.rs` — **Heap-quick sort**: the base hybrid.
- `heap_quick_sort_optimized.rs` — **Optimised heap-quick sort**: reduced overhead in the partition/heap interplay.
- `heap_quick_sort_optimized_tmp.rs` — **Optimised (tmp)**: experimental variant using temporary storage for further optimisation.
