# shell_sequences

Gap sequence generators for shell sort.

## What is a gap sequence?

Shell sort works by insertion-sorting elements that are `gap` positions apart, then shrinking the gap until it reaches 1 (at which point it's a standard insertion sort on a nearly-sorted array). The sequence of gap values — called the *gap sequence* — determines the sort's time complexity, constant factors, and visual behaviour.

A good gap sequence reduces inversions efficiently at large gaps so the final gap-1 pass does minimal work.

## The `GapSequence` trait

```rust
pub trait GapSequence {
    const NAME: &'static str;
    const BIG_O: &'static str;
    fn gaps(len: usize) -> Vec<usize>;
}
```

`gaps(len)` returns gap values in **descending** order (largest first) so shell sort can iterate them directly.

## Sequences

| Type | Sequence | Complexity | Notes |
|---|---|---|---|
| `Classic` | n/2, n/4, ..., 1 | O(N^2) | Shell's original (1959). Simple but poor worst case. |
| `Knuth` | 1, 4, 13, 40, ... (3k+1) | O(N^(3/2)) | Popular in textbooks. Good practical performance. |
| `Hibbard` | 1, 3, 7, 15, 31, ... (2^k - 1) | O(N^(3/2)) | Avoids the power-of-2 pathology of the classic sequence. |
| `Sedgewick` | 1, 8, 23, 77, 281, ... | O(N^(4/3)) | Sedgewick 1986. One of the best theoretically analysed sequences. |
| `SedgewickBranching` | 1, 5, 19, 41, 109, ... | O(N^(4/3)) | Sedgewick 1982. Alternating even/odd formulas. |
| `Ciura` | 1, 4, 10, 23, 57, 132, 301, 701, ... | ~O(N log N) | Empirically optimised (2001). Extended beyond 701 with ×2.25 multiplier. |
| `Tokuda` | 1, 4, 9, 20, 46, 103, ... | ~O(N^(4/3)) | Tokuda's empirically-derived sequence. Very good in practice. |
| `Pratt` | 1, 2, 3, 4, 6, 8, 9, 12, ... (2^p × 3^q) | O(N log^2 N) | Provably optimal for shell sort. Many more gaps than other sequences — trades pass count for guaranteed complexity. |
| `Optimized256` | 84, 25, 1 | ~O(N^1.5) | Pre-computed optimal gaps for arrays up to 256 elements. |
