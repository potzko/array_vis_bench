# rotate_merge_sorts (legacy)

Legacy in-place rotation merge sort implementations. These merge two sorted halves by rotating elements into position rather than using an auxiliary buffer.

## Variants

- `rotate_merge_sort.rs` — Top-down recursive rotation merge sort.
- `rotate_merge_sort_bottom_up.rs` — Iterative bottom-up rotation merge sort.
- `rotate_merge_sort_bottom_up_optimized.rs` — Bottom-up with early-exit and reduced rotation overhead.
- `rotate_merge_sort_optimized.rs` — Top-down with small-sort cutoff and early-exit.
- `utils.rs` — Shared rotation-merge helper (uses triple-reversal rotation).

## Superseded by

The new `merge_sorts/rotation.rs` + `rotation_merge.rs` module covers the same ground with a generic design that supports 11 different rotation algorithms and pluggable merge strategies (naive vs symMerge).
