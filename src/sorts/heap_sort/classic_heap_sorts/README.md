# classic_heap_sorts

Standard heap sorts with different branching factors (arity).

## What varies: branching factor

A binary heap (the textbook version) gives each node 2 children. Increasing the branching factor makes the tree shallower (fewer levels to sift through) but requires more comparisons per level (must find the maximum among all children). The trade-off affects cache behaviour and comparison count.

## Variants

- `heap_sort_classic.rs` — **Binary heap** (base-2). The standard version: each sift-down compares 2 children. O(N log2 N) comparisons.
- `base_3_heap.rs` — **Ternary heap** (base-3). Each sift-down compares 3 children. Shallower tree, ~5% fewer swaps in practice, but more comparisons per level.
- `base_16_heap.rs` — **16-ary heap**. Very shallow tree. More comparisons per level but better locality for the top few levels.
- `base_256_heap.rs` — **256-ary heap**. Extreme branching factor — practically flat. An experiment to see where the comparison overhead overwhelms the reduced tree depth.
