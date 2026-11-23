# `src`

Source of `merge_sort_lib`. The crate README one level up says what it is for.

- `auxiliary_merge.rs`: Auxiliary-array merges, the "normal" textbook merge that uses an external scratch buffer, in contrast to the in-place rotation-based merges in...
- `bottom_up.rs`: the crate's source.
- `lib.rs`: Merge sort family, classic (TopDown, BottomUp, Naive, Natural) and rotation-based merge sorts.
- `mask_merge.rs`: `merge_window`, the stable merge of a window that fits a bit mask held in local variables, an origin mask and the cycles that apply it.
- `naive.rs`: the crate's source.
- `natural.rs`: the crate's source.
- `rotation.rs`: the crate's source.
- `rotation_merge.rs`: the crate's source.
- `test_util.rs`: test helpers, the keyed `Item`, a tally of moves and comparisons and a replay of the logged moves.
- `top_down.rs`: the crate's source.
- `utils.rs`: the crate's source.
