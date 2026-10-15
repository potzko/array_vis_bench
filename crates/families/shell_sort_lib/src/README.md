# `src`

Source of `shell_sort_lib`. The crate README one level up says what it is for.

- `lib.rs`: Shell sort family, `ShellSort` + `ShellSortOrdered` generic over a `sequences::GapSequence`, with nine concrete sequences registered.
- `registration.rs`: Per-(algorithm x sequence) registration for shell sort.
- `sequences.rs`: the crate's source.
- `shell_sort.rs`: the crate's source.
- `shell_sort_ordered.rs`: the crate's source.
