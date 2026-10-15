# `src`

Source of `small_sort_deferred_insertion`. The crate README one level up says what it is for.

- `lib.rs`: Deferred-insertion small sort: quicksort stops at sub-arrays of length <= N and lets a single final insertion-sort pass (using strategy `S`) clean up.
