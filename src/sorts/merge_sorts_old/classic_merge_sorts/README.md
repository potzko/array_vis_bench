# classic_merge_sorts (legacy)

Legacy auxiliary-buffer merge sort implementations. These are the original hand-written versions, each with hardcoded merge logic.

## Variants

- `classic_merge_sort.rs` — Textbook top-down merge sort. Recursively splits, allocates a temp buffer, merges back.
- `merge_sort_bottom_up.rs` — Iterative bottom-up: doubles the block size each pass, merging adjacent pairs.
- `merge_sort_bottom_up_optimized.rs` — Bottom-up with reduced copy overhead (reuses a single buffer across passes).
- `merge_sort_optimized.rs` — Top-down with insertion-sort cutoff for small subarrays.
- `merge_sort_outside_lists.rs` — Merges into separate output lists rather than back into the original array.
- `utils.rs` — Shared two-way merge helper.

## Superseded by

The new `merge_sorts/` module (`top_down.rs`, `bottom_up.rs`, etc.) covers the same ground with a generic, parameterised design.
