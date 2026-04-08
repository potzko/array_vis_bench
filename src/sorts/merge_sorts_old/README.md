# merge_sorts_old

Legacy merge sort implementations from before the current `merge_sorts/` module was written. These are the original hand-written versions — each is a standalone sort with its own merge logic, without the parameterised trait system (`SmallSort`, `RotationMerge`, etc.) used by the new module.

Kept for reference and comparison. Not actively maintained.

## Subfolders

### `classic_merge_sorts/`
Merge sorts that use an auxiliary buffer for merging:
- `classic_merge_sort.rs` — textbook top-down merge sort.
- `merge_sort_bottom_up.rs` — iterative bottom-up variant.
- `merge_sort_bottom_up_optimized.rs` — bottom-up with early-exit and reduced copies.
- `merge_sort_optimized.rs` — top-down with optimisations (insertion-sort cutoff, etc.).
- `merge_sort_outside_lists.rs` — variant that merges into external lists.
- `utils.rs` — shared merge helper.

### `rotate_merge_sorts/`
Merge sorts that merge in-place using rotation:
- `rotate_merge_sort.rs` — top-down with rotation-based merge.
- `rotate_merge_sort_bottom_up.rs` — iterative bottom-up rotation merge.
- `rotate_merge_sort_bottom_up_optimized.rs` — bottom-up with optimisations.
- `rotate_merge_sort_optimized.rs` — top-down with optimisations.
- `utils.rs` — shared rotation merge helper.
