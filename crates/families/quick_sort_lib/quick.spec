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
  slot     a PartPivot piv_first
  slot     b PartPivot piv_middle
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
# `Pivoted<scheme, pivot>` / `DualPivoted<pivot>` are self-pivoting partitions
# (`N_PIVOTS = 0`). They are emitted BOTH as first-class Partition KIND entries (via
# `kinds.avbs`) AND slotted as the quick driver's `part`. The single/dual split bakes
# arity into the type; `pk_dual`'s DISTINCT `DualPivoted` head lets a query reference
# the dual composite unambiguously (both would otherwise share the `Pivoted` head).
# Together: 5×6 single (30, PartPivot now carries `last`) + 1×37 dual (37) = 67
# PartitionKind variants. They ALSO provide `SelectPartition` (quick-select's broader
# slot role, which additionally admits the pivotless heap-extract partitions). Quick
# curates `last` back out in its query, staying at its historical 51-of-67.
component pk_single
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind SelectPartition
  category Partition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot  PartPivot
end

component pk_dual
  type     DualPivoted<{pivot}>
  label    dual<{pivot}>
  provides PartitionKind SelectPartition
  category Partition
  uses     quick_sort_lib::DualPivoted
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

# ═══════════ BEAP-SORT'S DUAL-PIVOT SELECTORS (independent role) ═══════════
# beap-sort reuses the CombinedSelector / NintherDualPivot TYPES with its OWN role
# (BeapDualPivot), so it is unaffected by the unified partition system. (Quick-select
# no longer needs separate dual-selector components — it draws its duals from the
# shared PartPivotDual population in quick_select.avbs.)
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

# quick-select's dual selectors are no longer their own components: under the
# unified partition system quick-select builds its 4 curated dual composites
# (first/first, middle/middle, first/last, ninther) directly from the shared
# `CombinedSelector` / `NintherDualPivot` types in its own query (quick_select.avbs).
