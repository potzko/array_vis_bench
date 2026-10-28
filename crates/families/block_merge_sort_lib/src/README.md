# `src`

Source of `block_merge_sort_lib`. The crate README one level up says what it is for.

- `lib.rs`: `BlockMergeSort` and the whole sort.
- `rolling.rs`: `roll_merge`, the block merge that rolls blocks of the left run through the right run.
- `stash.rs`: `Stash`, where the left run of a local merge waits.
- `test_util.rs`: the item type, generator and comparison helpers of the module tests.
