# shell_sorts

Shell sort variants -- parameterised over gap sequences and traversal strategies.

## How shell sort works

Shell sort generalises insertion sort by comparing elements that are far apart, then progressively reducing the gap. For each gap value `g`, it insertion-sorts every `g`-spaced subsequence. When the gap reaches 1, the array is nearly sorted and a final insertion-sort pass finishes in nearly linear time.

The choice of gap sequence determines the worst-case complexity (from O(N^2) down to O(N log^2 N)) and the practical constant factors.

## Variants

### Shell sort (`shell_sort.rs`)

The standard version: for each gap, sweep left-to-right and insertion-sort each element into its `g`-spaced subsequence. Generic over `Seq: GapSequence` -- see [shell_sequences/README.md](../../utils/shell_sequences/README.md) for available sequences.

### Shell sort ordered (`shell_sort_ordered.rs`)

Same algorithm, different traversal: fully sorts one `g`-spaced subsequence (start, start+g, start+2g, ...) before moving to the next start offset. Same asymptotic behaviour but visually distinct access pattern.

## Registration

All variants self-register at startup via a `#[ctor]` in `combinations.rs`. The `sequences.rs` file defines a `GAP_SEQUENCES` distributed slice where each gap sequence registers its shell-sort and shell-sort-ordered variants.

## Files

- `shell_sort.rs` -- generic `ShellSort<Seq>`.
- `shell_sort_ordered.rs` -- generic `ShellSortOrdered<Seq>`.
- `sequences.rs` -- `GAP_SEQUENCES` slice and per-sequence registration entries.
- `combinations.rs` -- `#[ctor]` that registers all variants into `SORT_REGISTRY`.
- `classic_shell_sorts/` -- legacy hand-written shell sort variants (pre-generic system). See [classic_shell_sorts/README.md](classic_shell_sorts/README.md).
