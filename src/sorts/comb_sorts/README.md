# comb_sorts

Comb sort variants — an improvement on bubble sort that uses shrinking gaps to eliminate "turtles" (small values near the end).

## How comb sort works

Like bubble sort, comb sort compares and swaps pairs — but instead of always comparing adjacent elements, it starts with a large gap and shrinks it by a *shrink factor* each pass. When the gap reaches 1, it finishes with standard bubble sort passes until no swaps remain.

The classic shrink factor is 1.3 (empirically optimal). Different gap sequences produce different constant factors and visual patterns.

## Variants

- `comb_sort.rs` — **CombSort**: takes a pre-computed gap sequence and runs one forward pass per gap, then converges with gap-1 passes.
- `comb_sort_ratio.rs` — **CombSortRatio**: generates gaps from a configurable shrink ratio (e.g. 1.3, 1.5, etc.).
- `comb_classic.rs` — pre-built classic variant with shrink factor 1.3.
- `comb_random_gaps.rs` — experimental variant using randomly generated gap sequences.

## Registration

- `sequences.rs` — `COMB_SEQUENCES` distributed slice with one entry per shrink factor.
- `combinations.rs` — generates and registers all variant combinations.

## Status

Compiled but commented out of the active dispatch in `sorts/mod.rs`, pending migration to `sort_family!`.
