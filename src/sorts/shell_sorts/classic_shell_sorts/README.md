# classic_shell_sorts

Legacy hand-written shell sort variants from before the generic `ShellSort<Seq>` system was introduced. Each file is a standalone shell sort with a hardcoded gap sequence.

These are the pre-generic versions — the generic system in the parent folder (`shell_sort.rs` + `sequences.rs`) now covers the same ground more concisely. These remain compiled for backwards compatibility and because some contain additional experiments not yet replicated in the generic system.

## Variants

- `shell_classic.rs` — Shell's original sequence (n/2, n/4, ..., 1).
- `shell_hibbard.rs` — Hibbard sequence (2^k - 1).
- `shell_knuth.rs` — Knuth sequence (3k + 1).
- `shell_sedgewick.rs` — Sedgewick 1986 sequence.
- `shell_sedgewick_branching.rs` — Sedgewick 1982 branching sequence.
- `shell_classic_ordered_insertion.rs` — Classic sequence with ordered (per-subsequence) insertion sort traversal.
- `shell_sedgewick_ordered_insertion.rs` — Sedgewick sequence with ordered traversal.
- `shell_classic_dissonance.rs` — Classic sequence with an experimental "dissonance" access pattern.
- `shell_optimized_256_elements.rs` — Pre-computed optimal gaps for arrays up to 256 elements.
