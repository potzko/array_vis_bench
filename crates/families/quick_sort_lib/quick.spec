# Quick-sort family catalog fragment — OWNED by this crate.
#
# Gathered by `spec_catalog` via `[package.metadata.array_vis_bench] spec =
# "quick.spec"`. The quick driver slots ONE self-pivoting partition — the
# `PartitionKind` composite `Pivoted<scheme, pivot>` — so the pivot nests UNDER
# the partition (one `part` axis), there is no separate top-level pivot slot, and
# no query-level pivot↔partition coupling.
#
# ONE partition role-system: quick reuses the SAME `PartitionKind` partitions the
# first-class Partition kind emits (`kinds.avbs`). The old quick-only `Partition`
# / `Pivot` roles are gone; `PartScheme` / `PartPivot` (+ their dual siblings) are
# the single source.
#
# ARITY COUPLING (structural, baked into the composite, no query refinement): the
# single composite `pk_single` pairs a single scheme (`PartScheme`) with a single
# pivot (`PartPivot`); the dual composite `pk_dual` pairs the dual scheme
# (`PartSchemeDual`) with a dual selector (`PartPivotDual`). No component mixes
# arities, so an arity-illegal pairing is unrepresentable.

# ════════════════════════════ DUAL PIVOT SELECTORS ═══════════════════════
# The dual selectors of the unified partition system (role `PartPivotDual`).
# `combined` composes two single `PartPivot`s → 5 × 5 = 25 pairs.
component combined
  type     CombinedSelector<{a}, {b}>
  label    combined<{a}, {b}>
  provides PartPivotDual
  uses     quick_sort_lib::CombinedSelector
  slot     a PartPivot pkp_first
  slot     b PartPivot pkp_middle
end

# The baked "ninther thirds" dual selector (1 variant).
component ninther_dual
  type     NintherDualPivot
  label    ninther
  provides PartPivotDual
  uses     quick_sort_lib::NintherDualPivot
end

# ════════════════════════════ DUAL PARTITION SCHEME ══════════════════════
component pks_dual
  type     DualPivotPartition
  label    dual
  provides PartSchemeDual
  uses     quick_sort_lib::DualPivotPartition
end

# ════════════════════ FIRST-CLASS PARTITION COMPOSITES ════════════════════
# `Pivoted<scheme, pivot>` is a self-pivoting partition (`N_PIVOTS = 0`). These
# are emitted BOTH as first-class Partition KIND entries (via `kinds.avbs`) AND
# slotted as the quick driver's `part`. The single/dual split bakes arity into the
# type. Together: 5×5 single (25) + 1×26 dual (26) = 51 PartitionKind variants.
component pk_single
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind
  category Partition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot  PartPivot
end

component pk_dual
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind
  category Partition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartSchemeDual
  slot     pivot  PartPivotDual
end

# ════════════════════════ SLOTTED DRIVERS ════════════════════════════════
# One `part` slot (the `PartitionKind` composite) + small sort. The pivot lives
# inside `part`, so the label/picker nest it under the partition.
component quick_sort
  type     CanonicalQuickSort<{part}, {small_sort}>
  label    quick sort<part: {part}, small: {small_sort}>
  provides Sort
  category Sort
  menu     quick sorts
  uses     quick_sort_lib::CanonicalQuickSort
  slot     part PartitionKind
  slot     small_sort SmallSort
end

component deferred_quick_sort
  type     DeferredCanonicalQuickSort<{part}, {small_sort}>
  label    deferred quick sort<part: {part}, small: {small_sort}>
  provides Sort
  category Sort
  menu     deferred quick sorts
  uses     quick_sort_lib::DeferredCanonicalQuickSort
  slot     part PartitionKind
  slot     small_sort DeferredSmallSort
end

# ═══════════ OTHER FAMILIES' DUAL-PIVOT SELECTORS (independent roles) ═══════
# beap-sort + quick-select reuse the CombinedSelector / NintherDualPivot TYPES
# with their OWN roles (BeapDualPivot / QSPivotDual), so they are unaffected by
# the quick partition unification above.
component beap_dps_first
  type     CombinedSelector<FirstElement, FirstElement>
  label    first
  provides BeapDualPivot
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement
end

component beap_dps_middle
  type     CombinedSelector<MiddleElement, MiddleElement>
  label    middle
  provides BeapDualPivot
  uses     quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end

component beap_dps_ninther
  type     NintherDualPivot
  label    ninther 1/3 + 2/3
  provides BeapDualPivot
  uses     quick_sort_lib::NintherDualPivot
end

component qs_dp_first_first
  type     CombinedSelector<FirstElement, FirstElement>
  label    first / first
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement
end

component qs_dp_mid_mid
  type     CombinedSelector<MiddleElement, MiddleElement>
  label    middle / middle
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end

component qs_dp_first_last
  type     CombinedSelector<FirstElement, LastElement>
  label    first / last
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::CombinedSelector pivot_first::FirstElement pivot_last::LastElement
end

component qs_dp_ninther
  type     NintherDualPivot
  label    ninther 1/3 + 2/3
  provides QSPivot QSPivotDual
  uses     quick_sort_lib::NintherDualPivot
end

component qs_part_dual
  type     DualPivotPartition
  label    dual pivot
  provides QSPartition
  project  pivot QSPivotDual
  uses     quick_sort_lib::DualPivotPartition
end
