# `src`

Source of `rod_sort_lib`. The crate README one level up says what it is for.

- `lib.rs`: Rod sort family, `RodSort<S, M>` plus the `BranchingStrategy` trait and its six concrete strategies, and three `RodMerge` variants (insertion, aux, tournament).
- `gaps.rs`: `RodGaps<S>`, a strategy's gap chain as a Shell sort gap sequence.
- `merge.rs`: Final-merge strategy for `super::rod_sort::RodSort`.
- `rod_sort.rs`: the crate's source.
- `shell_branching.rs`: the crate's source.
- `testing.rs`: helpers shared by the unit tests.
