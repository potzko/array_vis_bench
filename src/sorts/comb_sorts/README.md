# comb_sorts

Comb sort variants — an improvement on bubble sort that uses shrinking gaps to eliminate "turtles" (small values near the end).

## How comb sort works

Like bubble sort, comb sort compares and swaps pairs — but instead of always comparing adjacent elements, it starts with a large gap and shrinks it by a *shrink factor* each pass. When the gap reaches 1, it finishes with standard bubble sort passes until no swaps remain.

The classic shrink factor is 1.3 (empirically optimal). Different gap sequences produce different constant factors and visual patterns.

## Variants

- `comb_sort.rs` — **CombSort**: takes a pre-computed gap sequence and runs one forward pass per gap, then converges with gap-1 passes.
- `comb_sort_ratio.rs` — **CombSortRatio<NUM, DEN>**: generic over the shrink ratio (1.3, √2, φ, 4/3, 11/8, 5/4 are all registered). The `family!` invocation enumerates the ratio set into the algorithm registry.

## Registration

- `sequences.rs` — `COMB_SEQUENCES` distributed slice that the per-ratio
  registration entries live in. Drives the `combo_codegen::family!`
  expansion in `comb_sort_ratio.rs`.
- `register_sequences.rs` — `#[ctor]` that loops over `COMB_SEQUENCES`
  and adds each to the navigation tree under `sorts/comb sorts/`.
