# `src`

Source of `tree_sort_lib`. The crate README one level up says what it is for.

- `lib.rs`: the crate root, re-exports and the shared test input.
- `nodes.rs`: `Nodes`, the node store, its `Layout`s (`Packed`, `Halves`, `Whole`) and the link operations every tree shares.
- `tree_sort.rs`: `TreeSort<B>`, the driver, and `Tree`, the role `B` fills.
- `avl.rs`: `Avl`, the height balanced tree.
- `naive.rs`: `Naive`, the tree that never rebalances.
