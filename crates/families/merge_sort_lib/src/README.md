# `src`

Source of `merge_sort_lib`. The crate README one level up says what it is for.

- `auxiliary_merge.rs`: Auxiliary-array merges, the "normal" textbook merge that uses an external scratch buffer, in contrast to the in-place rotation-based merges in...
- `bottom_up.rs`: the crate's source.
- `lib.rs`: Merge sort family, classic (TopDown, BottomUp, Naive, Natural) and rotation-based merge sorts.
- `mask_merge.rs`: `MaskRotationMerge`, a stable merge with no scratch of its own, trimmed to a window and merged by `merge_window`, an origin mask and the cycles that apply it, split by rotations when the window is too long. `MaskMerge` is its `MergeStrategy`.
- `naive.rs`: the crate's source.
- `natural.rs`: the crate's source.
- `rotation.rs`: the crate's source.
- `rotation_merge.rs`: `NaiveRotationMerge`, `SmallerSideRotationMerge`, which is symMerge over a rotation, and `SmallerSideMerge`, its `MergeStrategy`.
- `test_util.rs`: test helpers, the keyed `Item`, a tally of moves and comparisons and a replay of the logged moves.
- `top_down.rs`: the crate's source.
- `utils.rs`: the crate's source.
