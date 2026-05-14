# insertion_sorts

Insertion sort — the fundamental O(N^2) sort that many faster algorithms use as a building block for small subarrays.

## How insertion sort works

Scan left to right. For each element, shift it leftward past all larger elements until it finds its correct position. After processing element `i`, the prefix `arr[0..=i]` is sorted.

Insertion sort is:
- **Stable** — equal elements keep their original order.
- **Adaptive** — O(N) on already-sorted input (each element just checks one comparison and stays put).
- **Low overhead** — no recursion, no auxiliary memory, tiny constant factors.

These properties make it the preferred "small-sort" finisher for merge sort, timsort, and shell sort, where it handles subarrays of size ~16-64 after the main algorithm has reduced disorder.

## Files

- `insertion_sort.rs` — standard insertion sort with `SortLogger` instrumentation. Provides both generic `sort` and dyn-compatible `sort_dyn`.

## Registration

Single-leaf `combo_codegen::family!` invocation in `insertion_sort.rs`.
