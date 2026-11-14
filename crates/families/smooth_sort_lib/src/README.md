# `src`

Source of `smooth_sort_lib`. The crate README one level up says what it is for.

- `lib.rs`: the crate root, and the modules it re-exports.
- `shape.rs`: the tree shapes, `Leonardo` and `Perfect`, and the `Forest` of orders a heap is made of.
- `tree.rs`: the comparison and the moves on the logical positions of a heap, and the two subtrees of a tree.
- `sift.rs`: the sift of one tree.
- `heap.rs`: the operations on a forest, rectifying the roots, adding an element and removing the last.
- `build.rs`: how the forest is first made.
- `smooth_sort.rs`: `SmoothSort`, the heap algorithm.
- `tests.rs`: the tests of every combination.
