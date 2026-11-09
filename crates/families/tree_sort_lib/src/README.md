# `src`

Source of `tree_sort_lib`. The crate README one level up says what it is for.

- `lib.rs`: the crate root, re-exports and the shared test input.
- `nodes.rs`: `Nodes`, the node store, its `Layout`s (`Packed`, `Halves`, `Whole`) and the link operations every tree shares.
- `tree_sort.rs`: `TreeSort<B>`, the driver, and `Tree`, the role `B` fills.
- `avl.rs`: `Avl<SLACK>`, the height balanced tree whose siblings may differ by `SLACK` levels, the classic AVL tree at 1.
- `fixed_avl.rs`: `FixedAvl`, the classic AVL tree written for slack 1 alone, a few percent faster than `Avl<1>` on small arrays.
- `naive.rs`: `Naive`, the tree that never rebalances.
