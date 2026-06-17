# Comb-sort family catalog fragment — OWNED by this crate, gathered by
# `spec_catalog` via `[package.metadata.array_vis_bench] spec = "comb.spec"`.
#
# Single-axis: the `CombSortOf<{ratio}>` driver (unique type-head + inherent
# `sort`) ranges over the `CombRatio` shrink factors. The bare `CombSortRatio<N,D>`
# types all share the head `CombSortRatio`, so they can't be queried directly —
# but as named slot-fillers under the CombRatio role they enumerate cleanly.

component comb_sort
  type     CombSortOf<{ratio}>
  label    comb sort<{ratio}>
  provides Sort
  category Sort
  menu     comb sorts
  uses     comb_sort_lib::CombSortOf
  slot     ratio CombRatio comb_13
end

# ── Shrink factors (CombRatio = reciprocal NUM/DEN) ──────────────────────────
component comb_13
  type     CombSortRatio<10, 13>
  label    1.3
  provides CombRatio
  uses     comb_sort_lib::CombSortRatio
end

component comb_sqrt2
  type     CombSortRatio<70, 99>
  label    √2 ≈ 1.414
  provides CombRatio
  uses     comb_sort_lib::CombSortRatio
end

component comb_phi
  type     CombSortRatio<55, 89>
  label    φ ≈ 1.618
  provides CombRatio
  uses     comb_sort_lib::CombSortRatio
end

component comb_four_thirds
  type     CombSortRatio<3, 4>
  label    4/3
  provides CombRatio
  uses     comb_sort_lib::CombSortRatio
end

component comb_eleven_eighths
  type     CombSortRatio<8, 11>
  label    11/8
  provides CombRatio
  uses     comb_sort_lib::CombSortRatio
end

component comb_five_fourths
  type     CombSortRatio<4, 5>
  label    5/4
  provides CombRatio
  uses     comb_sort_lib::CombSortRatio
end
